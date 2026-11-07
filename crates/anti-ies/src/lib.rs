//! Núcleo experimental do anti-IES. Não afirma compatibilidade sem evidência.
//!
//! As respostas iniciais não imitam páginas nem dados do SEI.

/// Resultado observável de uma rota, independente da mensagem exibida.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Estado {
    Ok,
    RequisicaoInvalida,
    MetodoNaoPermitido,
    NaoEncontrado,
    NaoImplementado,
}

impl Estado {
    /// Status HTTP correspondente ao resultado tipado.
    #[must_use]
    pub const fn codigo_http(self) -> u16 {
        match self {
            Self::Ok => 200,
            Self::RequisicaoInvalida => 400,
            Self::MetodoNaoPermitido => 405,
            Self::NaoEncontrado => 404,
            Self::NaoImplementado => 501,
        }
    }

    /// Frase do protocolo HTTP, sem incluir informações da requisição.
    #[must_use]
    pub const fn motivo_http(self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::RequisicaoInvalida => "Bad Request",
            Self::MetodoNaoPermitido => "Method Not Allowed",
            Self::NaoEncontrado => "Not Found",
            Self::NaoImplementado => "Not Implemented",
        }
    }
}

/// Resposta de teste, sem cookies, URLs assinadas ou identificadores reais.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Resposta {
    pub estado: Estado,
    pub tipo_conteudo: &'static str,
    pub corpo: &'static str,
}

impl Resposta {
    #[must_use]
    pub const fn nova(estado: Estado) -> Self {
        let corpo = match estado {
            Estado::Ok => r#"{"status":"ok","servidor":"anti-ies"}"#,
            Estado::RequisicaoInvalida => r#"{"erro":"requisicao_invalida"}"#,
            Estado::MetodoNaoPermitido => r#"{"erro":"metodo_nao_permitido"}"#,
            Estado::NaoEncontrado => r#"{"erro":"rota_nao_encontrada"}"#,
            Estado::NaoImplementado => r#"{"erro":"operacao_sei_nao_implementada"}"#,
        };
        Self {
            estado,
            tipo_conteudo: "application/json; charset=utf-8",
            corpo,
        }
    }
}

/// Rotas mínimas: identidade do teste e recusa explícita de operações SEI.
///
/// A compatibilidade de cada ação só será acrescentada após uma cassette
/// saneada e um teste diferencial com o IES.
#[must_use]
pub fn despachar(metodo: &str, alvo: &str) -> Resposta {
    let estado = if alvo == "/__anti_ies/health" {
        if metodo == "GET" {
            Estado::Ok
        } else {
            Estado::MetodoNaoPermitido
        }
    } else if eh_controlador_sei(alvo) {
        Estado::NaoImplementado
    } else {
        Estado::NaoEncontrado
    };
    Resposta::nova(estado)
}

fn eh_controlador_sei(alvo: &str) -> bool {
    let caminho = alvo.split_once('?').map_or(alvo, |(caminho, _)| caminho);
    caminho.ends_with("/controlador.php")
}
