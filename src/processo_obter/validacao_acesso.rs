//! Validação de acesso com senha privada, somente para a modal efetivamente oferecida.
//! O formulário da modal ainda é hipótese: não há POST observado em cassette GO.
use super::abertura::{ArvoreAberta, Contexto, Lida, Rede, fora_de_palpite};
use super::{EnvioDoProcesso, ErroProcessoObter as E};
use crate::arvore_processo::{
    ErroTelaDoProcesso, endereco::conferir_endereco_relativo, ler_tela_do_processo,
};
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDeSessao, OperacaoDeFormulario,
    ResultadoDoEnvio, TelaServida, derivar_submissao, enviar_submissao_via,
};
use crate::pagina_do_controle::codificacao_da_tela;
use crate::sessao_login::Credenciais;
use std::path::PathBuf;

const VALIDAR_ACESSO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "usuario_validar_acesso",
    formulario: "frmValidarAcesso",
    envio: EnvioDoFormulario::BotaoDeEnvio("sbmValidar"),
    campos_ocultos_exigidos: &[],
    campos_ocultos_reescritos: &[],
    campos_usuario: &["pwdSenha"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
        o_que: EstadoDeSessao::AcessoAoProcessoValidado,
    },
};

/// Localiza a URL da única modal e confere ação, identidade e assinatura.
/// Nenhuma URL assinada é impressa nem inventada.
fn link_modal(html: &str, contexto: &Contexto<'_>, id: &str) -> Result<String, E> {
    let recusada = || E::Tela(ErroTelaDoProcesso::ProcessoExigeValidacaoDeAcesso);
    let mut fragmentos = html.split("infraAbrirJanelaModal('");
    let _ = fragmentos.next();
    let trecho = fragmentos.next().ok_or_else(recusada)?;
    if fragmentos.next().is_some() {
        return Err(recusada());
    }
    let (href, _) = trecho.split_once("',").ok_or_else(recusada)?;
    conferir_endereco_relativo(&contexto.instalacao, href, VALIDAR_ACESSO.acao, id)
        .map_err(|_| recusada())
}

/// Uma submissão e nenhuma tentativa automática posterior; a releitura ocorre
/// no chamador mesmo quando o POST responde 302.
pub(super) fn validar<V: EnvioDoProcesso>(
    rede: &mut Rede<'_, V>,
    contexto: &Contexto<'_>,
    html: &str,
    id: &str,
) -> Result<(), E> {
    let caminho = std::env::var_os("IES_CREDENCIAIS_FILE")
        .filter(|caminho| !caminho.is_empty())
        .ok_or(E::Tela(ErroTelaDoProcesso::ProcessoExigeValidacaoDeAcesso))?;
    let credenciais = Credenciais::ler(&PathBuf::from(caminho))?;
    let url = link_modal(html, contexto, id)?;
    let (html, mime) = match rede.get(&url)? {
        Lida::Html { html, mime } => (html, mime),
        Lida::Redirecionamento(_) => {
            return Err(E::Tela(ErroTelaDoProcesso::ProcessoExigeValidacaoDeAcesso));
        }
        Lida::Login => return Err(E::SessaoEncerrada(crate::erro_publico::SessaoEncerrada)),
        Lida::Falha(falha) => return Err(E::Transporte(falha)),
    };
    let tela = TelaServida {
        url: &url,
        html: &html,
        codificacao: codificacao_da_tela(&mime).ok_or(E::CharsetDoControleNaoSuportado)?,
    };
    let submissao = derivar_submissao(
        &VALIDAR_ACESSO,
        &contexto.instalacao,
        &tela,
        contexto.unidade_pedida.unwrap_or(""),
        &[("pwdSenha", credenciais.senha())],
    )?;
    let _ = enviar_submissao_via(submissao, contexto.cookie, |s, c| {
        rede.requisicoes.registrar();
        rede.envio.post(s, c)
    })?;
    Ok(())
}

/// Depois de um POST único, exige uma nova leitura independente do trabalho
/// e da árvore; se a exigência persistir, não repete a submissão.
pub(super) fn validar_e_abrir<V: EnvioDoProcesso>(
    rede: &mut Rede<'_, V>,
    contexto: &Contexto<'_>,
    trabalho: &str,
    html: &str,
    id: &str,
) -> Result<(ArvoreAberta, String), E> {
    validar(rede, contexto, html, id)?;
    let (html, mime) = match rede.get(trabalho)? {
        Lida::Html { html, mime } => (html, mime),
        Lida::Redirecionamento(local) => return Err(fora_de_palpite(local.as_deref())),
        Lida::Login => return Err(E::SessaoEncerrada(crate::erro_publico::SessaoEncerrada)),
        Lida::Falha(falha) => return Err(E::Transporte(falha)),
    };
    let tela = ler_tela_do_processo(&contexto.instalacao, &html, id, contexto.unidade_pedida)?;
    rede.abrir_da_tela(contexto, trabalho, (&html, &mime), tela, id)
}
