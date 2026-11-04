//! A tela "Conclusão de Processo" (forma `tela`, SEI 4.1 e 5.0) e a
//! resposta das duas formas (contracts/cli/processo-concluir.md).
//!
//! Evidência (cassettes públicas do Todos, UFRJ 4.1.5, PEN2 4.1.5, GO
//! 5.0.3, PEN3 e PEN4 5.0.4, telas idênticas): `form#frmDesentranharDocumento`
//! `method="post"`; ocultos `hdnInfraTipoPagina`, `hdnIdProtocolos` (o id) e
//! `hdnIdProcedimentoDestino`; `sbmSalvar`; `rdoConcluir` `S` marcado e `V`;
//! a reabertura programada vazia. O navegador envia `rdoConcluir=S` e os
//! dois textos vazios (teste do Todos que mediu o envio na UFRJ); o SEI
//! responde 302 para `arvore_visualizar` (ou `principal`), da conclusão do
//! mesmo processo.
use super::erro::ErroTelaDeConclusao as E;
use crate::arvore_processo::ACAO_DE_CONCLUIR;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDoProcesso,
    OperacaoDeFormulario, ResultadoDoEnvio,
};

pub(crate) const FORMULARIO_DE_CONCLUSAO: &str = "frmDesentranharDocumento";
pub(crate) const BOTAO_SALVAR: &str = "sbmSalvar";
const OCULTOS: [&str; 3] = [
    "hdnInfraTipoPagina",
    "hdnIdProtocolos",
    "hdnIdProcedimentoDestino",
];
const OPCAO: &str = "rdoConcluir";
const SOMENTE_CONCLUIR: &str = "S";
const PRAZO: &str = "rdoPrazoReaberturaProgramada";
const DATA: &str = "txtPrazoReaberturaProgramada";
const DIAS: &str = "txtDiasReaberturaProgramada";
const UTEIS: &str = "chkSinDiasUteisReaberturaProgramada";
const PROCESSOS: &str = "selProcedimentos";

/// Salvar a tela de conclusão: janela sem barra da unidade, aberta pelo
/// link da barra da árvore da tela do processo conferida.
pub(crate) const CONCLUIR_PROCESSO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: ACAO_DE_CONCLUIR,
    formulario: FORMULARIO_DE_CONCLUSAO,
    envio: EnvioDoFormulario::BotaoDeEnvio(BOTAO_SALVAR),
    campos_ocultos_exigidos: &OCULTOS,
    campos_ocultos_reescritos: &[],
    campos_usuario: &[OPCAO, DATA, DIAS],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::AndamentoAbertoNaUnidade,
    },
};

/// Os pares do usuário: "Somente concluir" e os textos vazios.
pub(crate) const CAMPOS_DO_ENVIO: [(&str, &str); 3] =
    [(OPCAO, SOMENTE_CONCLUIR), (DATA, ""), (DIAS, "")];

/// Confere a tela servida antes de qualquer POST.
pub(crate) fn conferir_tela_de_conclusao(html: &str, id: &str) -> Result<(), E> {
    let _ = (html, id, PRAZO, UTEIS, PROCESSOS);
    todo!("RED")
}

/// O 302 do POST é o esperado: `controlador.php?acao=arvore_visualizar`
/// ou `acao=principal`, com `acao_origem=procedimento_concluir` e o id.
pub(crate) fn redirecionamento_esperado(localizacao: Option<&str>, id: &str) -> bool {
    let _ = (localizacao, id);
    todo!("RED")
}
