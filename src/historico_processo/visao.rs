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
    OperacaoDeFormulario, ResultadoDoEnvio,
};

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

/// Confere, literalmente, a função servida que o POST emula (sem executar
/// JS). Qual visão veio depois do POST é conferido na resposta
/// (`VisaoDoHistoricoDiferenteDaPedida`); a âncora não é conferida.
pub(crate) fn conferir_regra_de_ver_historico(html: &str) -> Result<(), super::ErroAndamentos> {
    let linhas: Vec<&str> = html.lines().map(str::trim).collect();
    if REGRA_DE_VER_HISTORICO
        .iter()
        .all(|regra| linhas.contains(regra))
    {
        Ok(())
    } else {
        Err(super::ErroAndamentos::RegraDeVerHistoricoAusenteNaPagina)
    }
}
