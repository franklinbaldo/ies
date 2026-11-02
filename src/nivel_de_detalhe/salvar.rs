//! POST de salvar do Nível de detalhe (`sbmSalvar`), declarado pela própria
//! operação `unidade.definir_colunas` (contracts/cli/unidade-definir-colunas.md).
//!
//! Evidência (ensaio do proprietário na PEN1 5.0.4, treinamento, 02/11/2026):
//! o navegador envia só `hdnInfraTipoPagina` (como servido), `sbmSalvar` com
//! o valor servido e as caixas marcadas com valor `on` (sem atributo
//! `value`); a resposta é 200 sem redirect, uma página que fecha o modal,
//! e a mudança vale na leitura seguinte. Não há marcador de resultado
//! observado: o efeito é conferido por um GET da mesma tela.
use super::tela::{ACAO_DO_NIVEL_DE_DETALHE, ErroNivelDeDetalhe, FORMULARIO_DO_NIVEL_DE_DETALHE};
use super::{ColunaDoDetalhe, PlanoDasColunas};
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, OperacaoDeFormulario,
    PreferenciaDoUsuario, ResultadoDoEnvio,
};

/// `name` das 12 caixas, na ordem de [`ColunaDoDetalhe::TODAS`].
pub(crate) const CAMPOS_DAS_OPCOES: [&str; 12] = [
    "chkSinAtribuicao",
    "chkSinAnotacao",
    "chkSinTipoProcesso",
    "chkSinEspecificacao",
    "chkSinNivelPrioritarios",
    "chkSinInteressados",
    "chkSinObservacao",
    "chkSinControlePrazo",
    "chkSinRetornoDevolver",
    "chkSinRetornoAguardando",
    "chkSinUltimaMovimentacao",
    "chkSinMarcadores",
];

/// Oculto que o formulário publica e o POST repete como servido.
pub(crate) const OCULTO_DO_NIVEL_DE_DETALHE: &str = "hdnInfraTipoPagina";
pub(crate) const BOTAO_SALVAR: &str = "sbmSalvar";

/// Salvar a tela `procedimento_configurar_detalhe`. A tela é uma janela
/// sem barra da unidade, aberta pelo link da D já conferida.
pub(crate) const SALVAR_NIVEL_DE_DETALHE: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: ACAO_DO_NIVEL_DE_DETALHE,
    formulario: FORMULARIO_DO_NIVEL_DE_DETALHE,
    envio: EnvioDoFormulario::BotaoDeEnvio(BOTAO_SALVAR),
    campos_ocultos_exigidos: &[OCULTO_DO_NIVEL_DE_DETALHE],
    campos_ocultos_reescritos: &[],
    campos_usuario: &CAMPOS_DAS_OPCOES,
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraPreferenciaDoUsuario {
        o_que: PreferenciaDoUsuario::ColunasDaVisaoDetalhada,
    },
};

/// Valor que o navegador envia numa caixa marcada sem atributo `value`.
pub(crate) const VALOR_DA_CAIXA_MARCADA: &str = "on";

/// Pares do usuário para o alvo: uma caixa `on` por coluna marcada.
pub(crate) fn campos_do_envio(plano: &PlanoDasColunas) -> Vec<(&'static str, &'static str)> {
    let _ = plano;
    todo!("RED: campos do envio")
}

/// Recusa, antes de qualquer POST, controle enviável que o formulário
/// observado não tem: só `hdnInfraTipoPagina`, `sbmSalvar` e as caixas do
/// catálogo. Botões `button`/`reset` e controles `disabled` não são enviados.
pub(crate) fn conferir_controles_do_formulario(html: &str) -> Result<(), ErroNivelDeDetalhe> {
    let _ = (html, ColunaDoDetalhe::TODAS);
    todo!("RED: conferir controles")
}
