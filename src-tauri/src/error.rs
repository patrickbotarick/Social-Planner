use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AppError {
    pub code: &'static str,
    pub message: &'static str,
}

impl AppError {
    pub fn validation() -> Self {
        Self { code: "VALIDATION", message: "Informe título de até 200 caracteres, legenda de até 20.000 caracteres e mídias compatíveis: Feed 1 imagem, Carrossel 2–20 mídias, Reels 1 vídeo, Stories 1 mídia." }
    }
    pub fn media() -> Self {
        Self { code: "MEDIA_INVALID", message: "Arquivo incompatível ou danificado. Use JPEG, PNG, WebP ou MP4 com vídeo; imagens até 64 MB e 40 milhões de pixels, vídeos até 512 MB." }
    }
    pub fn file() -> Self {
        Self { code: "MEDIA_IO", message: "Não foi possível ler ou copiar a mídia. Confira se o arquivo existe, está acessível e há espaço no disco. O original permanece intacto." }
    }
    pub fn not_found() -> Self {
        Self {
            code: "NOT_FOUND",
            message: "A publicação ou mídia não está disponível. Atualize a biblioteca.",
        }
    }
    pub fn conflict() -> Self {
        Self {
            code: "EDIT_CONFLICT",
            message: "A publicação mudou desde que foi aberta. Reabra para editar a versão atual.",
        }
    }
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
