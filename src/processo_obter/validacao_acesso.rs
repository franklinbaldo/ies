//! Revalidacao de senha de processo: modalidade sintetica do BDD-633.
//! O GET da modal e o formulario nao estao em cassettes; nenhuma URL
//! assinada e montada ou adivinhada e nenhum segredo entra no Debug.
use super::abertura::{Contexto, Lida, Rede};
use super::{EnvioDoProcesso, ErroProcessoObter as E};
use crate::arvore_processo::conferir_endereco_relativo;
use crate::formulario_web::{
    CodificacaoDaTela, ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario,
    ErroSubmissaoFormulario, EstadoDeSessao, OperacaoDeFormulario, RespostaDoEnvio,
    ResultadoDoEnvio, TelaServida, derivar_submissao, enviar_submissao_via,
};
use crate::sessao_login::{Credenciais, ErroLogin};
use crate::transporte::StatusHttpRecusado;
use std::path::PathBuf;

const OPERACAO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "usuario_validar_acesso",
    formulario: "frmValidarAcesso",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &[],
    campos_ocultos_reescritos: &[],
    campos_usuario: &["txtSenha"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
        o_que: EstadoDeSessao::AcessoAoProcessoRevalidado,
    },
};

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(super) enum ErroValidacaoAcesso {
    ModalNaoReconhecida,
    Credenciais(#[from] ErroLogin),
    Formulario(#[from] ErroSubmissaoFormulario),
    RedirecionamentoSemProva,
}

impl ErroValidacaoAcesso {
    pub(super) fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Formulario(erro) => erro.status_http(),
            _ => None,
        }
    }
}

/// A modal so pode usar o link oferecido pela tela ja obtida.
/// Nem o URL nem o segredo da senha podem ser colocados em logs.
fn link_oferecido(contexto: &Contexto<'_>, id: &str, html: &str) -> Result<String, ErroValidacaoAcesso> {
    const MARCADOR: &str = "controlador.php?acao=usuario_validar_acesso&";
    let mut ocorrencias = html.match_indices(MARCADOR);
    let (inicio, _) = ocorrencias.next().ok_or(ErroValidacaoAcesso::ModalNaoReconhecida)?;
    if ocorrencias.next().is_some() {
        return Err(ErroValidacaoAcesso::ModalNaoReconhecida);
    }
    let restante = &html[inicio..];
    let fim = restante.find('\'').ok_or(ErroValidacaoAcesso::ModalNaoReconhecida)?;
    let relativo = &restante[..fim];
    conferir_endereco_relativo(&contexto.instalacao, relativo, OPERACAO.acao, id)
        .map_err(|_| ErroValidacaoAcesso::ModalNaoReconhecida)
}

/// Apenas uma tentativa de POST na modal. Sucesso provisório exige nova
/// leitura independente do processo; nunca repete o POST.
pub(super) fn validar<V: EnvioDoProcesso>(
    rede: &mut Rede<'_, V>,
    contexto: &Contexto<'_>,
    id: &str,
    tela_do_trabalho: &str,
) -> Result<(), E> {
    let Some(caminho) = std::env::var_os("IES_CREDENCIAIS_FILE").filter(|c| !c.is_empty()) else {
        return Err(E::Tela(crate::arvore_processo::ErroTelaDoProcesso::ProcessoExigeValidacaoDeAcesso));
    };
    let url = link_oferecido(contexto, id, tela_do_trabalho)?;
    let credenciais = Credenciais::ler(&PathBuf::from(caminho)).map_err(ErroValidacaoAcesso::from)?;
    let (html, codificacao) = match rede.get(&url)? {
        Lida::Html { html, mime } => {
            let codificacao = crate::pagina_do_controle::codificacao_da_tela(&mime)
                .ok_or(ErroValidacaoAcesso::ModalNaoReconhecida)?;
            (html, codificacao)
        }
        Lida::Redirecionamento(_) | Lida::Login => {
            return Err(ErroValidacaoAcesso::RedirecionamentoSemProva.into());
        }
        Lida::Falha(falha) => return Err(E::Transporte(falha)),
    };
    let tela = TelaServida {
        url: &url,
        html: &html,
        codificacao,
    };
    let submissao = derivar_submissao(
        &OPERACAO,
        &contexto.instalacao,
        &tela,
        contexto.unidade_pedida.unwrap_or(""),
        &[("txtSenha", credenciais.senha())],
    )?;
    rede.requisicoes.registrar();
    let resposta = enviar_submissao_via(submissao, contexto.cookie, |s, c| rede.envio.post(s, c))?;
    match resposta {
        RespostaDoEnvio::Html(_) => Ok(()),
        RespostaDoEnvio::Redirecionamento(_) => {
            Err(ErroValidacaoAcesso::RedirecionamentoSemProva.into())
        }
    }
}
