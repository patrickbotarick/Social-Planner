use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AppError {
    pub code: &'static str,
    pub message: &'static str,
}

impl AppError {
    pub fn database() -> Self {
        Self {
            code: "DATABASE_UNAVAILABLE",
            message: "Não foi possível acessar o banco local. Preserve os dados e verifique as permissões.",
        }
    }

    pub fn integrity() -> Self {
        Self {
            code: "DATABASE_INTEGRITY",
            message: "A integridade do banco não pôde ser confirmada. Preserve os arquivos.",
        }
    }

    pub fn schema() -> Self {
        Self {
            code: "SCHEMA_UNSUPPORTED",
            message: "A versão ou o histórico do banco não é compatível com este aplicativo.",
        }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(_: rusqlite::Error) -> Self {
        // Não propagar SQL, caminhos, dados editoriais ou credenciais ao log/IPC.
        Self::database()
    }
}
