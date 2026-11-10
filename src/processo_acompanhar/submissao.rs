//! Derivação pura do POST de inclusão de Acompanhamento Especial (#254).
//! Não faz I/O: o formulário e a URL devem vir da MESMA resposta servida.
//! Caso do Todos `tests/test_processo_acompanhar_010.py` @323eb01.
use super::fluxo::ErroPrevia;
use super::{escolher_grupo, grupos_do_cadastro};
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDoProcesso,
    OperacaoDeFormulario, ResultadoDoEnvio, SubmissaoDerivada, TelaServida, derivar_submissao,
};
use crate::instalacao::Instalacao;

/// Cadastro observado na `UnB`: botão de envio e oculto do processo originais.
/// O primitivo recusa action fora da instalação, hash inválido, botão ausente,
/// duplicidade de controles e campos não ofertados.
const INCLUIR: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "acompanhamento_cadastrar",
    formulario: "frmAcompanhamentoCadastro",
    envio: EnvioDoFormulario::BotaoDeEnvio("sbmCadastrarAcompanhamento"),
    campos_ocultos_exigidos: &["hdnIdProtocolo"],
    campos_ocultos_reescritos: &[],
    campos_usuario: &["selGrupoAcompanhamento", "txaObservacao"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::AcompanhamentoEspecial,
    },
};

/// Constrói exatamente UM POST, mantendo ocultos e botão do formulário,
/// grupo por nome exato e observação na codificação da tela. Não o envia.
pub(crate) fn derivar_inclusao(
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    id_procedimento: &str,
    unidade: &str,
    grupo: Option<&str>,
    observacao: &str,
) -> Result<SubmissaoDerivada, ErroPrevia> {
    let grupos = grupos_do_cadastro(tela.html, id_procedimento)?;
    let escolhido = escolher_grupo(&grupos, grupo)?;
    derivar_submissao(
        &INCLUIR,
        instalacao,
        tela,
        unidade,
        &[
            ("selGrupoAcompanhamento", &escolhido.valor),
            ("txaObservacao", observacao),
        ],
    )
    .map_err(ErroPrevia::Formulario)
}

/// Exclusão pelo `submit()` do formulário observado na lista. Sem botão.
const EXCLUIR: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "acompanhamento_excluir",
    formulario: "frmGerenciarAcompanhamento",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &["hdnInfraItemId"],
    campos_ocultos_reescritos: &["hdnInfraItemId"],
    campos_usuario: &[],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::AcompanhamentoEspecial,
    },
};

/// A action assinada e o alvo foram conferidos na mesma tela servida.
pub(crate) fn derivar_exclusao(
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    unidade: &str,
    id_alvo: &str,
    action_observada: &str,
) -> Result<SubmissaoDerivada, ErroPrevia> {
    crate::formulario_web::derivar_submissao_com_action(
        &EXCLUIR,
        instalacao,
        tela,
        unidade,
        &[("hdnInfraItemId", id_alvo)],
        Some(action_observada),
    )
    .map_err(ErroPrevia::Formulario)
}
