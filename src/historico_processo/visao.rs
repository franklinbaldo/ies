//! "Ver histórico completo": `a#ancTipoHistorico` com
//! `onclick="verHistorico('P');"` e a função servida
//! `function verHistorico(valor){` →
//! `document.getElementById('hdnTipoHistorico').value = valor;` →
//! `document.getElementById('frmProcedimentoHistorico').submit();`
//! (`UnB` 4.0.9 e GO 5.0.3, 03/11/2026). O POST reenvia os ocultos como
//! servidos, só `hdnTipoHistorico` reescrito, sem botão. O botão
//! `btnAtualizarAndamento` (escrita) nunca participa: é `type=button`.
use super::ErroAndamentos as E;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDeSessao, MarcadorDeResultado,
    OperacaoDeFormulario, ResultadoDoEnvio,
};

pub(crate) const CAMPO_TIPO: &str = "hdnTipoHistorico";

/// `verHistorico('P')`: `submit()` por script; a página do histórico não
/// publica a unidade (conferida na tela de origem do link).
pub(crate) const VER_HISTORICO_COMPLETO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "procedimento_consultar_historico",
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

/// Confere, literalmente, a função servida e o link para a visão `P`.
pub(crate) fn conferir_regra_de_ver_historico(_html: &str) -> Result<(), E> {
    Err(E::RegraDeVerHistoricoNaoObservada)
}
