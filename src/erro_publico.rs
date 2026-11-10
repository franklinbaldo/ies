//! Fronteira pública dos erros ([contrato](../contracts/cli/erros.md)).
//!
//! O tipo é a verdade; o texto é só a codificação para o JSON. Cada erro é
//! declarado no módulo que o levanta e deriva [`Causa`]: o caminho
//! (`["transporte", "rede"]`) e a [`Acao`] saem da composição de tipos, sem
//! tabela manual de subcódigos. Nenhum código de domínio compara esses
//! textos: classifica-se por `match` nas variantes.
use crate::transporte::StatusHttpRecusado;
use serde_json::{Value, json};

pub(crate) use ies_derive::Causa;

#[allow(dead_code)] // consumido pelo piloto (processo concluir), na PR seguinte.
/// A decisão de recuperação que o agente pode tomar. Pequena e fechada:
/// é o contrato de decisão; a causa é diagnóstico.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Acao {
    /// Uso inválido: corrigir os argumentos e repetir.
    CorrigirArgumentos,
    /// A sessão foi encerrada: `ies login` e repetir.
    Relogar,
    /// A instalação bloqueou o acesso: não insistir; usar outra.
    TrocarInstalacao,
    /// A escrita foi enviada e a reconciliação não pôde ser lida: ler de
    /// novo (`processo`, `unidade gerados`) antes de qualquer repetição.
    ConferirEscrita,
    /// Nada a recuperar automaticamente: reportar a causa.
    Reportar,
}

#[allow(dead_code)] // consumido pelo piloto (processo concluir), na PR seguinte.
impl Acao {
    pub(crate) fn texto(self) -> &'static str {
        match self {
            Self::CorrigirArgumentos => "corrigir_argumentos",
            Self::Relogar => "relogar",
            Self::TrocarInstalacao => "trocar_instalacao",
            Self::ConferirEscrita => "conferir_escrita",
            Self::Reportar => "reportar",
        }
    }
}

/// Implementado por `#[derive(Causa)]`.
pub(crate) trait Causa {
    /// Acrescenta o caminho da causa, do erro externo ao mais interno.
    fn caminho(&self, c: &mut Vec<&'static str>);
    /// O nome do fato: a variante mais interna que não é só uma `fonte`.
    fn folha(&self) -> &'static str;
    #[allow(dead_code)] // consumido pelo piloto (processo concluir), na PR seguinte.
    fn acao(&self) -> Acao;
    fn status_http(&self) -> Option<&StatusHttpRecusado>;
    /// A sessão do SEI acabou (`#[causa(sessao_encerrada)]`): decide o
    /// relogin das leituras pelo tipo.
    fn sessao_encerrada(&self) -> bool;
}

pub(crate) fn caminho_de(erro: &impl Causa) -> Vec<&'static str> {
    let mut caminho = Vec::new();
    erro.caminho(&mut caminho);
    caminho
}

/// O nome mais interno do caminho: o `subcodigo` publicado até a
/// migração terminar (obsoleto).
pub(crate) fn folha(erro: &impl Causa) -> &'static str {
    erro.folha()
}

#[allow(dead_code)] // consumido pelo piloto (processo concluir), na PR seguinte.
/// O que se sabe de uma escrita, por fatos: enviada ou não, e o que a
/// leitura posterior mostrou. Não há "efeito incerto": a incerteza é só a
/// ausência de leitura, e a causa dessa ausência vai em `causa`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Escrita {
    pub(crate) enviada: bool,
    /// `None`: nenhuma leitura posterior concluiu.
    pub(crate) aplicada: Option<bool>,
}

#[allow(dead_code)] // consumido pelo piloto (processo concluir), na PR seguinte.
/// O objeto JSON da falha. `subcodigo` e `codigo` seguem por
/// compatibilidade (obsoletos): `subcodigo` é a folha do caminho.
pub(crate) fn json_da_falha(
    operacao: &str,
    erro: &impl Causa,
    mensagem: &str,
    escrita: Option<&Escrita>,
) -> Value {
    let caminho = caminho_de(erro);
    let acao = erro.acao();
    let mut valor = json!({
        "codigo": if acao == Acao::ConferirEscrita { "efeito_incerto" } else { "fonte_invalida" },
        "subcodigo": folha(erro),
        "acao": acao.texto(),
        "operacao": operacao,
        "causa": caminho,
        "erro": mensagem,
        "retentavel": false
    });
    let objeto = valor.as_object_mut().expect("objeto");
    if let Some(escrita) = escrita {
        objeto.insert(
            "escrita".to_owned(),
            json!({ "enviada": escrita.enviada, "aplicada": escrita.aplicada }),
        );
    }
    if let Some(http) = erro.status_http() {
        if http.status != 200 {
            objeto.insert("status_http".to_owned(), json!(http.status));
        }
        if let Some(texto) = &http.mensagem_do_servidor {
            objeto.insert("mensagem_do_servidor".to_owned(), json!(texto));
        }
    }
    valor
}

#[allow(dead_code)] // consumido pelo piloto (processo concluir), na PR seguinte.
/// A linha humana: mensagem, caminho e, se houver, o status do SEI.
pub(crate) fn linha_da_falha(erro: &impl Causa, mensagem: &str) -> String {
    let caminho = caminho_de(erro).join("/");
    let base = format!("IES: {mensagem} [{caminho}]");
    match erro.status_http() {
        None => base,
        Some(StatusHttpRecusado {
            status,
            mensagem_do_servidor,
        }) => match mensagem_do_servidor {
            Some(texto) if *status == 200 => format!("{base} (SEI: {texto})"),
            Some(texto) => format!("{base} (HTTP {status}: {texto})"),
            None => format!("{base} (HTTP {status})"),
        },
    }
}

#[allow(dead_code)] // consumido pelo piloto (processo concluir), na PR seguinte.
/// Saída do processo: 3 só quando a escrita foi enviada e nenhuma leitura
/// mostrou o resultado; 1 nas demais falhas.
pub(crate) fn codigo_de_saida(erro: &impl Causa) -> u8 {
    if erro.acao() == Acao::ConferirEscrita {
        3
    } else {
        1
    }
}

#[cfg(test)]
#[path = "erro_publico/tests.rs"]
mod tests;
