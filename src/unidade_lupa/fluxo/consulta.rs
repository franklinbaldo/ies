//! Derivação do POST da lupa com a forma efetivamente servida.
//! Separado da navegação para manter cada peça menor que 20 KB.
use super::FalhaDaLupa;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDeSessao, OperacaoDeFormulario,
    ResultadoDoEnvio, SubmissaoDerivada, TelaServida, derivar_submissao,
};
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;

const CONSULTAR: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "unidade_selecionar_envio_processo",
    formulario: "frmUnidadeLista",
    envio: EnvioDoFormulario::BotaoDeEnvio("btnPesquisar"),
    campos_ocultos_exigidos: &["hdnInfraPaginaAtual"],
    campos_ocultos_reescritos: &["hdnInfraPaginaAtual"],
    campos_usuario: &[
        "selOrgaoUnidade",
        "txtSiglaUnidade",
        "txtDescricaoUnidade",
        "selInfraPaginacaoSuperior",
        "selInfraPaginacaoInferior",
    ],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
        o_que: EstadoDeSessao::PaginaOuFiltroLembrado,
    },
};

pub(super) fn consulta(
    instalacao: &Instalacao,
    url: &str,
    html: &str,
    mime: &str,
    pagina: usize,
) -> Result<SubmissaoDerivada, FalhaDaLupa> {
    let codificacao = codificacao_da_tela(mime).ok_or(FalhaDaLupa::CharsetInesperado)?;
    let indice = pagina.to_string();
    let tela = TelaServida {
        url,
        html,
        codificacao,
    };
    // Controles de paginação devem existir como o SEI os publica.
    // A biblioteca de formulário insere os ocultos fixos na ordem correta.
    derivar_submissao(
        &CONSULTAR,
        instalacao,
        &tela,
        "",
        &[
            ("selOrgaoUnidade", ""),
            ("txtSiglaUnidade", ""),
            ("txtDescricaoUnidade", ""),
            ("hdnInfraPaginaAtual", &indice),
            ("selInfraPaginacaoSuperior", &indice),
            ("selInfraPaginacaoInferior", &indice),
        ],
    )
    .map_err(|_| FalhaDaLupa::FormularioInesperado)
}

