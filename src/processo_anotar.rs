//! Prévia e registro único de anotação a partir do formulário servido.
//! A releitura da mesma identidade ocorre após o POST, sem retry.
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::MotivoAbertoEm;
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::formulario_web::{
    ErroSubmissaoFormulario, RespostaDoEnvio, TelaServida, consulta_do_controlador,
    derivar_submissao, enviar_submissao_via,
};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::CookieIlegivel;
use crate::processo_concluir::escrita::{
    Contador, FalhaDaTela, leitura, ler_cookie, obter_tela, siglas_abertas,
};
use crate::processo_enviar::REGISTRAR_ANOTACAO;
use crate::processo_obter::{
    EnvioDoProcesso, ErroProcessoObter, SessaoDoProcesso, abrir_para_escrever,
};
use crate::transporte::FalhaTransporte;
use crate::visao_do_controle::EnvioDoControle as _;

/// Intenção exclusiva de consulta, registro ou retirada.
pub(crate) enum Intencao<'a> {
    Consulta,
    Registro { texto: &'a str, confirmar: bool },
    Retirada { confirmar: bool },
}

pub(crate) struct Previa {
    pub(crate) nup: String,
    pub(crate) id_procedimento: String,
    pub(crate) unidade: String,
    pub(crate) texto: String,
    pub(crate) anotacao_antes: String,
    pub(crate) requisicoes: u8,
    pub(crate) alterado: bool,
}

/// `--texto` vazio ou só com espaços.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) struct TextoDaAnotacaoVazio;

/// `--texto`: o clap recusa o vazio (saída 2) antes da sessão.
pub(crate) fn texto_da_anotacao(texto: &str) -> Result<String, TextoDaAnotacaoVazio> {
    if texto.trim().is_empty() {
        Err(TextoDaAnotacaoVazio)
    } else {
        Ok(texto.to_owned())
    }
}

/// Um erro por causa; o diagnóstico é derivado do tipo Rust.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroAnotar {
    Abertura(#[from] ErroProcessoObter),
    LinkDasAnotacoes(#[from] crate::arvore_processo::ErroLinkDasAnotacoes),
    LeituraDaAnotacao(#[from] crate::processo_enviar::ErroLeituraDaAnotacao),
    Transporte(#[from] FalhaTransporte),
    Cookie(#[from] CookieIlegivel),
    TelaRedirecionada,
    SessaoEncerrada(#[source] crate::erro_publico::SessaoEncerrada),
    UnidadeNaoPublicada,
    SituacaoNaoLida(MotivoAbertoEm),
    ProcessoForaDaUnidade,
    AnotacaoNaoOferecida,
    AnotacaoJaPresente,
    SalvarNaoOferecido,
    OrigemNaoAutorizada,
    CharsetNaoSuportado,
    Formulario(#[from] ErroSubmissaoFormulario),
    NaoConfirmado,
}

/// A prévia não altera a anotação; confirmação tenta exatamente um POST.
pub(crate) fn executar<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    nup: &NumeroProcesso,
    intencao: &Intencao<'_>,
    unidade: Option<&str>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<Previa, ErroAnotar> {
    let (texto, confirmar, remover) = match intencao {
        Intencao::Consulta => ("", false, false),
        Intencao::Registro { texto, confirmar } => (*texto, *confirmar, false),
        Intencao::Retirada { confirmar } => ("", *confirmar, true),
    };
    let mut rede = Contador::novo(envio);
    let (aberto, acoes) =
        abrir_para_escrever(sessao, &leitura(nup, unidade, true), indice, &mut rede)?;
    let unidade = aberto.unidade.ok_or(ErroAnotar::UnidadeNaoPublicada)?;
    let abertas = siglas_abertas(&aberto.arvore.aberto_em).map_err(ErroAnotar::SituacaoNaoLida)?;
    if !abertas.contains(&unidade) {
        return Err(ErroAnotar::ProcessoForaDaUnidade);
    }
    let id = aberto.arvore.processo.id;
    let url = acoes.anotacoes?.ok_or(ErroAnotar::AnotacaoNaoOferecida)?;
    let cookie = ler_cookie(sessao)?;
    let (html, mime) = obter_tela(&mut rede, &cookie, &url).map_err(|falha| match falha {
        FalhaDaTela::Transporte(erro) => ErroAnotar::Transporte(erro),
        FalhaDaTela::Redirecionada => ErroAnotar::TelaRedirecionada,
        FalhaDaTela::SessaoEncerrada(crate::erro_publico::SessaoEncerrada) => {
            ErroAnotar::SessaoEncerrada(crate::erro_publico::SessaoEncerrada)
        }
    })?;
    let anotacao_antes = crate::processo_enviar::texto_anotacao(&html, &id)?;
    // Retirada com anotação comprovadamente vazia dispensa POST.
    let alterado = if confirmar && (!remover || !anotacao_antes.is_empty()) {
        if !remover && !anotacao_antes.trim().is_empty() {
            return Err(ErroAnotar::AnotacaoJaPresente);
        }
        if !crate::processo_enviar::salvar_oferecido(&html)? {
            return Err(ErroAnotar::SalvarNaoOferecido);
        }
        let instalacao =
            Instalacao::interpretar(sessao.origem).ok_or(ErroAnotar::OrigemNaoAutorizada)?;
        let codificacao = codificacao_da_tela(&mime).ok_or(ErroAnotar::CharsetNaoSuportado)?;
        let servida = TelaServida {
            url: &url,
            html: &html,
            codificacao,
        };
        let submissao = derivar_submissao(
            &REGISTRAR_ANOTACAO,
            &instalacao,
            &servida,
            None,
            &[("txaDescricao", if remover { "" } else { texto })],
        )?;
        let mut enviada = false;
        let resposta = enviar_submissao_via(submissao, sessao.cookie, |sub, cookie| {
            enviada = true;
            rede.post(sub, cookie)
        });
        let resposta_aceita = match resposta {
            Ok(RespostaDoEnvio::Redirecionamento(ref r)) => {
                redirecionamento_esperado(r.localizacao(), r.url_do_envio(), &id)
            }
            Ok(_) => false,
            Err(_) if enviada => false,
            Err(erro) => return Err(ErroAnotar::Formulario(erro)),
        };
        // A resposta recusada não prova autoria, mesmo se um GET posterior
        // encontrar o texto desejado. Não repetir o POST em nenhum caso.
        let relido = obter_tela(&mut rede, &cookie, &url)
            .ok()
            .and_then(|(html, _)| crate::processo_enviar::texto_anotacao(&html, &id).ok());
        if !resposta_aceita || relido.as_deref() != Some(if remover { "" } else { texto }) {
            return Err(ErroAnotar::NaoConfirmado);
        }
        true
    } else {
        false
    };
    Ok(Previa {
        nup: nup.as_str().to_owned(),
        id_procedimento: id,
        unidade,
        texto: texto.to_owned(),
        anotacao_antes,
        requisicoes: rede.feitas(),
        alterado,
    })
}

/// O 302 não é seguido. A identidade e a ação precisam concordar.
fn redirecionamento_esperado(localizacao: Option<&str>, url: &str, id: &str) -> bool {
    let Some(consulta) = localizacao.and_then(|l| consulta_do_controlador(l, url)) else {
        return false;
    };
    let Ok(consulta) = ConsultaDoControlador::ler(consulta) else {
        return false;
    };
    consulta.acao() == Ok(AcaoDoSei::ArvoreVisualizar)
        && consulta.acao_de_origem() == Ok(AcaoDoSei::AnotacaoRegistrar)
        && consulta.unico("id_procedimento") == Ok(id)
}
#[cfg(test)]
mod acervo;
