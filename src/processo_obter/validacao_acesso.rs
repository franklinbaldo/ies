//! Revalidacao de senha de processo: modalidade sintetica do BDD-633.
//! O GET da modal e o formulario nao estao em cassettes; nenhuma URL
//! assinada e montada ou adivinhada e nenhum segredo entra no Debug.
use super::abertura::{ArvoreAberta, Contexto, Lida, Rede};
use super::{EnvioDoProcesso, ErroProcessoObter};
use crate::arvore_processo::{conferir_endereco_relativo, ler_tela_do_processo};
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, ErroSubmissaoFormulario,
    EstadoDeSessao, OperacaoDeFormulario, RespostaDoEnvio, ResultadoDoEnvio, TelaServida,
    derivar_submissao, enviar_submissao_via,
};
use crate::sessao_login::{Credenciais, ErroLogin};
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
pub(crate) enum ErroValidacaoAcesso {
    ModalNaoReconhecida,
    Credenciais(#[from] ErroLogin),
    Formulario(#[from] ErroSubmissaoFormulario),
    RedirecionamentoSemProva,
    /// A sessao mudou depois do POST; nao reiniciar a operacao nem reenviar senha.
    ReleituraNaoConfirmada,
}

impl ErroValidacaoAcesso {
    pub(crate) fn sessao_encerrada(&self) -> bool {
        match self {
            Self::Credenciais(erro) => erro.sessao_encerrada(),
            Self::Formulario(erro) => erro.sessao_encerrada(),
            Self::ModalNaoReconhecida
            | Self::RedirecionamentoSemProva
            | Self::ReleituraNaoConfirmada => false,
        }
    }
}

/// A modal so pode usar o link oferecido pela tela ja obtida.
/// Nem o URL nem o segredo da senha podem ser colocados em logs.
fn link_oferecido(
    contexto: &Contexto<'_>,
    id: &str,
    html: &str,
) -> Result<String, ErroValidacaoAcesso> {
    const MARCADOR: &str = "controlador.php?acao=usuario_validar_acesso&";
    let mut ocorrencias = html.match_indices(MARCADOR);
    let (inicio, _) = ocorrencias
        .next()
        .ok_or(ErroValidacaoAcesso::ModalNaoReconhecida)?;
    if ocorrencias.next().is_some() {
        return Err(ErroValidacaoAcesso::ModalNaoReconhecida);
    }
    let (_, restante) = html.split_at(inicio);
    let (relativo, _) = restante
        .split_once('\'')
        .ok_or(ErroValidacaoAcesso::ModalNaoReconhecida)?;
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
) -> Result<(), ErroProcessoObter> {
    let Some(caminho) = std::env::var_os("IES_CREDENCIAIS_FILE").filter(|c| !c.is_empty()) else {
        return Err(ErroProcessoObter::Tela(
            crate::arvore_processo::ErroTelaDoProcesso::ProcessoExigeValidacaoDeAcesso,
        ));
    };
    let url = link_oferecido(contexto, id, tela_do_trabalho)?;
    let credenciais =
        Credenciais::ler(&PathBuf::from(caminho)).map_err(ErroValidacaoAcesso::from)?;
    let (html, codificacao) = match rede.get(&url)? {
        Lida::Html { html, mime } => {
            let codificacao = crate::pagina_do_controle::codificacao_da_tela(&mime)
                .ok_or(ErroValidacaoAcesso::ModalNaoReconhecida)?;
            (html, codificacao)
        }
        Lida::Redirecionamento(_) | Lida::Login => {
            return Err(ErroValidacaoAcesso::RedirecionamentoSemProva.into());
        }
        Lida::Falha(falha) => return Err(ErroProcessoObter::Transporte(falha)),
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
    )
    .map_err(ErroValidacaoAcesso::from)?;
    rede.requisicoes.registrar();
    let resposta = enviar_submissao_via(submissao, contexto.cookie, |s, c| rede.envio.post(s, c))
        .map_err(ErroValidacaoAcesso::from)?;
    // Nem HTML nem 302 comprovam a validação: ambos exigem a releitura
    // independente do trabalho e da árvore, sem seguir Location nem retry.
    match resposta {
        RespostaDoEnvio::Html(_) | RespostaDoEnvio::Redirecionamento(_) => Ok(()),
    }
}

/// Completa a leitura após a tela de trabalho; quando surge a modal,
/// envia exatamente um formulario e confere a abertura novamente.
/// O restante da cadeia de resolucao nao e repetido.
pub(super) fn abrir_trabalho<V: EnvioDoProcesso>(
    rede: &mut Rede<'_, V>,
    contexto: &Contexto<'_>,
    trabalho: &str,
    id: &str,
    (html, mime): (&str, &str),
) -> Result<(ArvoreAberta, String), ErroProcessoObter> {
    let tela = match ler_tela_do_processo(&contexto.instalacao, html, id, contexto.unidade_pedida) {
        Ok(tela) => tela,
        Err(crate::arvore_processo::ErroTelaDoProcesso::ProcessoExigeValidacaoDeAcesso) => {
            validar(rede, contexto, id, html)?;
            let (nova_tela, novo_mime) = match rede.get(trabalho)? {
                Lida::Html { html, mime } => (html, mime),
                Lida::Redirecionamento(_) | Lida::Login => {
                    return Err(ErroValidacaoAcesso::ReleituraNaoConfirmada.into());
                }
                Lida::Falha(falha) if falha.sessao_encerrada() => {
                    return Err(ErroValidacaoAcesso::ReleituraNaoConfirmada.into());
                }
                Lida::Falha(falha) => return Err(ErroProcessoObter::Transporte(falha)),
            };
            let validada = ler_tela_do_processo(
                &contexto.instalacao,
                &nova_tela,
                id,
                contexto.unidade_pedida,
            )?;
            let resultado =
                rede.abrir_da_tela(contexto, trabalho, (&nova_tela, &novo_mime), validada, id);
            if resultado
                .as_ref()
                .is_err_and(ErroProcessoObter::sessao_encerrada)
            {
                return Err(ErroValidacaoAcesso::ReleituraNaoConfirmada.into());
            }
            return resultado;
        }
        Err(erro) => return Err(ErroProcessoObter::Tela(erro)),
    };
    rede.abrir_da_tela(contexto, trabalho, (html, mime), tela, id)
}
