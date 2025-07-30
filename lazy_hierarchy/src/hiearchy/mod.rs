pub mod concrete;
pub mod lazy;
pub use concrete::ConcreteHiearchy as Concrete;
pub use lazy::AsGroupRef as Lazy;

#[cfg(test)]
mod tests;
