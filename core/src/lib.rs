pub mod profiler;

pub struct Agent {
    // Badly named
}

pub struct Config {}

pub struct Handle {}

pub struct Plan {}

pub struct Error {}

impl Client {
    pub fn new(_config: Config) -> Self {
        todo!()
    }

    pub async fn run() -> Handle {
        todo!()
    }
}

impl Handle {
    pub async fn enqueue(_plan: Plan) -> Result<(), Error> {
        todo!()
    }

    pub async fn halt() -> Result<(), Error> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(4, 4);
    }
}
