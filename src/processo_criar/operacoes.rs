//! Contratos dos três formulários da criação Web de processo.
use crate::acao_do_sei::AcaoDoSei;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDeSessao, EstadoDoProcesso,
    OperacaoDeFormulario, ResultadoDoEnvio,
};

pub(super) const FILTRO_TIPOS: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::ProcedimentoEscolherTipo,
    formulario: "frmProcedimentoEscolherTipo",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &["hdnFiltroTipoProcedimento"],
    campos_ocultos_reescritos: &["hdnFiltroTipoProcedimento"],
    campos_usuario: &[],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
        o_que: EstadoDeSessao::PaginaOuFiltroLembrado,
    },
};

pub(super) const ESCOLHER_TIPO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::ProcedimentoEscolherTipo,
    formulario: "frmProcedimentoEscolherTipo",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &["hdnIdTipoProcedimento"],
    campos_ocultos_reescritos: &["hdnIdTipoProcedimento"],
    campos_usuario: &[],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::ConsultaSemEfeitoObservado {
        evidencia: "docs/como-o-sei-funciona/inicio-do-processo.md: escolher(id) abre o cadastro",
    },
};

pub(super) const CADASTRAR: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::ProcedimentoGerar,
    formulario: "frmProcedimentoCadastro",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &[
        "hdnIdTipoProcedimento",
        "hdnFlagProcedimentoCadastro",
        "hdnAssuntos",
    ],
    campos_ocultos_reescritos: &[
        "hdnFlagProcedimentoCadastro",
        "hdnAssuntos",
        "hdnInteressadosProcedimento",
    ],
    campos_usuario: &["txtDescricao", "rdoNivelAcesso", "selHipoteseLegal"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::ProcessoCriadoNaUnidade,
    },
};
