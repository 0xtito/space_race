use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{self, Ident};

/// Macro to derive an EntityCommand to register and store one-shot systems.
/// Created by @eidloi and can be found at:
/// https://gist.github.com/eidloi/916d6e4706bdc7f961f808ced9ff7ff1
#[proc_macro_derive(EventHandler)]
pub fn event_handler_macro_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_event_handler_macro(&ast)
}

fn impl_event_handler_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let handler_name = name.to_string().clone() + "Handler";
    let handler_ident = Ident::new(handler_name.as_str(), Span::call_site());

    let gen = quote! {
        pub struct #handler_ident<Marker, F> {
            callback: F,
            _marker: PhantomData<Marker>,
        }

        impl<Marker, F> #handler_ident<Marker, F> {
            pub fn from(callback: F) -> Self {
                Self {
                    callback: callback,
                    _marker: PhantomData,
                }
            }
        }

        impl<Marker, F> EntityCommand for #handler_ident<Marker, F>
        where
            Marker: Send + 'static,
            F: Send + IntoSystem<(), (), Marker> + 'static,
        {
            fn apply(self, id: bevy::ecs::entity::Entity, world: &mut World) {
                let system_id = world.register_system(self.callback);
                world.entity_mut(id).insert(#name{
                    system_id,
                    active: false,
                });
            }
        }
    };
    gen.into()
}