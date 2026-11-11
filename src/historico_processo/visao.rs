//! "Ver histórico completo": `a#ancTipoHistorico` com
//! `onclick="verHistorico('P');"` e a função servida
//! `function verHistorico(valor){` →
//! `document.getElementById('hdnTipoHistorico').value = valor;` →
//! `document.getElementById('frmProcedimentoHistorico').submit();`
//! (`UnB` 4.0.9 e GO 5.0.3, 03/11/2026). O POST reenvia os ocultos como
//! servidos, só `hdnTipoHistorico` reescrito, sem botão. O botão
//! `btnAtualizarAndamento` (escrita) nunca participa: é `type=button`.
use crate::acao_do_sei::AcaoDoSei;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDeSessao, MarcadorDeResultado,
    OperacaoDeFormulario, ResultadoDoEnvio, decodificar_atributo,
};
use crate::html_atributos::{nos_com_id, valor_do_atributo};

pub(crate) const CAMPO_TIPO: &str = "hdnTipoHistorico";

/// `verHistorico('P')`: `submit()` por script; a página do histórico não
/// publica a unidade (conferida na tela de origem do link).
pub(crate) const VER_HISTORICO_COMPLETO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::ProcedimentoConsultarHistorico,
    formulario: "frmProcedimentoHistorico",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &[CAMPO_TIPO],
    campos_ocultos_reescritos: &[CAMPO_TIPO],
    campos_usuario: &[],
    resultado: ResultadoDoEnvio::Marcador(MarcadorDeResultado {
        elemento: "table",
        classe: "infraTable",
    }),
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    // Não medido se o servidor lembra a visão na sessão.
    efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
        o_que: EstadoDeSessao::PaginaOuFiltroLembrado,
    },
};

/// Linhas literais de `verHistorico(valor)` servidas (sem interpretar JS).
const REGRA_DE_VER_HISTORICO: [&str; 3] = [
    "function verHistorico(valor){",
    "document.getElementById('hdnTipoHistorico').value = valor;",
    "document.getElementById('frmProcedimentoHistorico').submit();",
];

/// Confere, literalmente, a função servida e o único link para a visão `P`.
pub(crate) fn conferir_regra_de_ver_historico(html: &str) -> Result<(), super::ErroAndamentos> {
    let linhas: Vec<&str> = html.lines().map(str::trim).collect();
    if !REGRA_DE_VER_HISTORICO
        .iter()
        .all(|regra| linhas.contains(regra))
    {
        return Err(super::ErroAndamentos::RegraDeVerHistoricoNaoObservada);
    }
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(super::ErroAndamentos::RegraDeVerHistoricoNaoInterpretada)?;
    let parser = dom.parser();
    let links = nos_com_id(&dom, "ancTipoHistorico").map_err(super::ErroAndamentos::Atributo)?;
    let [link] = links.as_slice() else {
        return Err(super::ErroAndamentos::RegraDeVerHistoricoNaoObservada);
    };
    let link = link
        .get(parser)
        .and_then(tl::Node::as_tag)
        .filter(|t| t.name().as_utf8_str().eq_ignore_ascii_case("a"))
        .ok_or(super::ErroAndamentos::RegraDeVerHistoricoNaoObservada)?;
    let onclick = valor_do_atributo(link, "onclick")
        .map_err(super::ErroAndamentos::Atributo)?
        .and_then(|bruto| decodificar_atributo(&bruto))
        .ok_or(super::ErroAndamentos::RegraDeVerHistoricoNaoObservada)?;
    if onclick != "verHistorico('P');" {
        return Err(super::ErroAndamentos::RegraDeVerHistoricoNaoObservada);
    }
    Ok(())
}
