pub trait IdCommandHandler{
    type Input;
    type Output;

    fn handle(input: Input, store: impl IdStore) -> Result<Output>;
}