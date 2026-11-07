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
    DadosInsuficientes,
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
            Self::DadosInsuficientes => 503,
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
            Self::DadosInsuficientes => "Service Unavailable",
        }
    }
}

/// Resposta de teste, sem cookies, URLs assinadas ou identificadores reais.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Resposta {
    pub estado: Estado,
    pub tipo_conteudo: &'static str,
    pub corpo: String,
}

impl Resposta {
    #[must_use]
    pub fn nova(estado: Estado) -> Self {
        let corpo = match estado {
            Estado::Ok => r#"{"status":"ok","servidor":"anti-ies"}"#,
            Estado::RequisicaoInvalida => r#"{"erro":"requisicao_invalida"}"#,
            Estado::MetodoNaoPermitido => r#"{"erro":"metodo_nao_permitido"}"#,
            Estado::NaoEncontrado => r#"{"erro":"rota_nao_encontrada"}"#,
            Estado::NaoImplementado => r#"{"erro":"operacao_sei_nao_implementada"}"#,
            Estado::DadosInsuficientes => r#"{"erro":"observacao_insuficiente"}"#,
        };
        Self {
            estado,
            tipo_conteudo: "application/json; charset=utf-8",
            corpo: corpo.to_owned(),
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

use iesei::observacao::{
    Cobertura, FormaDoControle, ObservacaoDoControle, ProcessoNaLista, TabelaObservada,
};
use std::fmt::Write as _;

fn escapar_html(texto: &str) -> String {
    texto
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn linhas_html(tabela: &TabelaObservada<ProcessoNaLista>, html: &mut String) {
    for processo in &tabela.linhas {
        let _ = write!(
            html,
            "<tr id='P{}'><td>linha</td><td><a class='processoVisualizado' \
             href='controlador.php?acao=procedimento_trabalhar&amp;id_procedimento={}'>\
             {}</a></td><td>Ofício</td></tr>",
            processo.id,
            processo.id,
            escapar_html(&processo.numero)
        );
    }
}

fn legenda(tabela: &TabelaObservada<ProcessoNaLista>) -> String {
    let total = tabela.total_declarado.unwrap_or(tabela.linhas.len());
    match tabela.faixa_observada {
        Some(faixa) => format!("{total} registros - {} a {}:", faixa.inicio(), faixa.fim()),
        None => format!("{total} registros:"),
    }
}

fn renderizar_controle(observado: &ObservacaoDoControle<ProcessoNaLista>) -> Option<String> {
    if !observado.suficiente_para_servir() {
        return None;
    }
    let mut html = format!(
        "<html><body><a id='lnkInfraUnidade'>{}</a>",
        escapar_html(observado.unidade.as_str())
    );
    match &observado.forma {
        FormaDoControle::Resumida { recebidos, gerados } => {
            html.push_str("<input id='hdnTipoVisualizacao' value='R'>");
            for (id, categoria) in [
                ("tblProcessosRecebidos", recebidos),
                ("tblProcessosGerados", gerados),
            ] {
                let Cobertura::Observada(tabela) = categoria else {
                    return None;
                };
                let _ = write!(
                    html,
                    "<table id='{id}'><caption>{}</caption>",
                    legenda(tabela)
                );
                linhas_html(tabela, &mut html);
                html.push_str("</table>");
            }
        }
        FormaDoControle::Detalhada { processos } => {
            let Cobertura::Observada(tabela) = processos else {
                return None;
            };
            html.push_str(
                "<input id='hdnTipoVisualizacao' value='D'>\
                 <input id='hdnRecebidosHashCriterios' value='r'>\
                 <input id='hdnGeradosHashCriterios' value='g'>",
            );
            let _ = write!(
                html,
                "<table id='tblProcessosDetalhado'><caption>({})</caption>\
                 <tr><th></th><th>Processo</th><th>Tipo</th></tr>",
                legenda(tabela)
            );
            linhas_html(tabela, &mut html);
            html.push_str("</table>");
        }
    }
    html.push_str("</body></html>");
    Some(html)
}

/// Somente a rota do Controle observada é habilitada. Dados desconhecidos
/// geram HTTP 503, nunca listas vazias ou sucesso fictício.
#[must_use]
pub fn despachar_com_controle(
    metodo: &str,
    alvo: &str,
    observado: Option<&ObservacaoDoControle<ProcessoNaLista>>,
) -> Resposta {
    if metodo != "GET"
        || !eh_controlador_sei(alvo)
        || !alvo.ends_with("?acao=procedimento_controlar")
    {
        return despachar(metodo, alvo);
    }
    let Some(html) = observado.and_then(renderizar_controle) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    Resposta {
        estado: Estado::Ok,
        tipo_conteudo: "text/html; charset=utf-8",
        corpo: html,
    }
}
