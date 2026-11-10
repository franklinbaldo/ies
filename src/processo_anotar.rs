//! Prévia e registro único de anotação a partir do formulário servido.
//! A releitura da mesma identidade ocorre após o POST, sem retry.
use crate::arvore_processo::MotivoAbertoEm;
use crate::erro_html::ErroHtml;
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

pub(crate) struct Previa {
    pub(crate) nup: String,
    pub(crate) id_procedimento: String,
    pub(crate) unidade: String,
    pub(crate) texto: String,
    pub(crate) anotacao_antes: String,
    pub(crate) requisicoes: u8,
    pub(crate) alterado: bool,
}

/// Uma variante por causa; o diagnóstico é derivado do tipo Rust.
#[derive(Debug, crate::erro_publico::Causa)]
pub(crate) enum Erro {
    #[causa(interna)]
    Abertura(ErroProcessoObter),
    /// Barra ou tela das anotações fora da forma observada (`onde` diz qual).
    #[causa(transparente)]
    Html(ErroHtml),
    #[causa(interna)]
    Transporte(FalhaTransporte),
    #[causa(interna)]
    Cookie(CookieIlegivel),
    TelaRedirecionada,
    #[causa(sessao_encerrada)]
    SessaoEncerrada,
    UnidadeNaoPublicada,
    SituacaoNaoLida(MotivoAbertoEm),
    ProcessoForaDaUnidade,
    AnotacaoNaoOferecida,
    AnotacaoJaPresente,
    SalvarNaoOferecido,
    OrigemNaoAutorizada,
    CharsetNaoSuportado,
    #[causa(interna)]
    Formulario(ErroSubmissaoFormulario),
    EfeitoIncerto,
}

/// A prévia não altera a anotação; confirmação tenta exatamente um POST.
pub(crate) fn executar<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    nup: &NumeroProcesso,
    texto: &str,
    confirmar: bool,
    unidade: Option<&str>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<Previa, Erro> {
    let mut rede = Contador::novo(envio);
    let (aberto, acoes) =
        abrir_para_escrever(sessao, &leitura(nup, unidade, true), indice, &mut rede)?;
    let unidade = aberto.unidade.ok_or(Erro::UnidadeNaoPublicada)?;
    let abertas = siglas_abertas(&aberto.arvore.aberto_em).map_err(Erro::SituacaoNaoLida)?;
    if !abertas.contains(&unidade) {
        return Err(Erro::ProcessoForaDaUnidade);
    }
    let id = aberto.arvore.processo.id;
    let url = acoes.anotacoes?.ok_or(Erro::AnotacaoNaoOferecida)?;
    let cookie = ler_cookie(sessao)?;
    let (html, mime) = obter_tela(&mut rede, &cookie, &url).map_err(|falha| match falha {
        FalhaDaTela::Transporte(erro) => Erro::Transporte(erro),
        FalhaDaTela::Redirecionada => Erro::TelaRedirecionada,
        FalhaDaTela::SessaoEncerrada => Erro::SessaoEncerrada,
    })?;
    let anotacao_antes = crate::processo_enviar::texto_anotacao(&html, &id)?;
    let mut alterado = false;
    if confirmar {
        if !anotacao_antes.trim().is_empty() {
            return Err(Erro::AnotacaoJaPresente);
        }
        if !crate::processo_enviar::salvar_oferecido(&html) {
            return Err(Erro::SalvarNaoOferecido);
        }
        let instalacao = Instalacao::interpretar(sessao.origem).ok_or(Erro::OrigemNaoAutorizada)?;
        let codificacao = codificacao_da_tela(&mime).ok_or(Erro::CharsetNaoSuportado)?;
        let servida = TelaServida {
            url: &url,
            html: &html,
            codificacao,
        };
        let submissao = derivar_submissao(
            &REGISTRAR_ANOTACAO,
            &instalacao,
            &servida,
            "",
            &[("txaDescricao", texto)],
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
            Err(erro) => return Err(Erro::Formulario(erro)),
        };
        // A resposta recusada não prova autoria, mesmo se um GET posterior
        // encontrar o texto desejado. Não repetir o POST em nenhum caso.
        let relido = obter_tela(&mut rede, &cookie, &url)
            .ok()
            .and_then(|(html, _)| crate::processo_enviar::texto_anotacao(&html, &id).ok());
        if !resposta_aceita || relido.as_deref() != Some(texto) {
            return Err(Erro::EfeitoIncerto);
        }
        alterado = true;
    }
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
    let valores = |chave: &str| -> Vec<&str> {
        consulta
            .split('&')
            .filter_map(|par| par.split_once('='))
            .filter(|(nome, _)| *nome == chave)
            .map(|(_, valor)| valor)
            .collect()
    };
    valores("acao") == ["arvore_visualizar"]
        && valores("acao_origem") == ["anotacao_registrar"]
        && valores("id_procedimento") == [id]
}
