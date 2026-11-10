//! Prévia Web de anexação: abre principal, lê a tela e consulta somente
//! a identidade do anexado e sua condição de assinatura. Não grava nada.
//! POST de escrita apenas após prévia e confirmação, com releitura de prova.
use super::{Destino, decidir, ler_confirmacao, ler_destino, ler_tela};
use crate::arvore_processo::ErroAnexarNaBarra;
use crate::formulario_web::{
    ConsultaAjax, EfeitoAvaliado, ErroSubmissaoFormulario, TelaServida, derivar_consulta_ajax,
};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::CookieIlegivel;
use crate::processo_concluir::escrita::{Contador, FalhaDaTela, leitura, ler_cookie, obter_tela};
use crate::processo_obter::{
    EnvioDoProcesso, ErroProcessoObter, SessaoDoProcesso, abrir_para_escrever, obter_processo,
};
use crate::transporte::{FalhaTransporte, decodificar_resposta_xml};
use crate::visao_do_controle::EnvioDoControle as _;

const IDENTIFICAR: ConsultaAjax = ConsultaAjax {
    acao_ajax: "protocolo_RI1023",
    campos: &["idProcedimento"],
    efeito: EfeitoAvaliado::ConsultaSemEfeitoObservado {
        evidencia: "Todos/src/todos/anexacao.py @323eb01, UnB 4.0.9",
    },
};
const CONFERIR: ConsultaAjax = ConsultaAjax {
    acao_ajax: "procedimento_anexacao_verificar",
    campos: &["idProcedimento"],
    efeito: EfeitoAvaliado::ConsultaSemEfeitoObservado {
        evidencia: "Todos/src/todos/anexacao.py @323eb01, UnB 4.0.9",
    },
};

#[derive(Clone, Copy, Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum EtapaNaoComprovada {
    RespostaDoPost,
    ListaAposPost,
    ArvoreDoPrincipal,
}
#[derive(Debug)]
pub(crate) enum FalhaAnexar {
    /// O cookie privado da sessão não pôde ser lido.
    Cookie(CookieIlegivel),
    EfeitoIncerto(EtapaNaoComprovada),
    Abertura(ErroProcessoObter),
    Barra(ErroAnexarNaBarra),
    LinkNaoPublicado,
    Tela(FalhaTransporte),
    TelaRedirecionada,
    SessaoEncerrada,
    Estrutura(&'static str),
    ConsultaDerivacao(ErroSubmissaoFormulario),
    ConsultaTransporte(FalhaTransporte),
    ConsultaResposta(FalhaTransporte),
}
impl FalhaAnexar {
    // Apenas as recusas legadas possuem subcódigo durante a migração.
    // A falha nova é publicada pela variante tipada, não por tabela textual.
    pub(crate) fn subcodigo_legado(&self) -> Option<&'static str> {
        match self {
            Self::Cookie(e) => Some(crate::erro_publico::folha(e)),
            Self::EfeitoIncerto(_) => None,
            Self::Abertura(erro) => Some(erro.subcodigo()),
            Self::Barra(erro) => Some(erro.subcodigo()),
            Self::LinkNaoPublicado => Some("anexacao_nao_publicada_na_barra"),
            Self::Tela(erro) | Self::ConsultaTransporte(erro) | Self::ConsultaResposta(erro) => {
                Some(erro.subcodigo())
            }
            Self::TelaRedirecionada => Some("tela_de_anexacao_redirecionada"),
            Self::SessaoEncerrada => Some("sessao_encerrada_rode_ies_login"),
            Self::Estrutura(codigo) => Some(codigo),
            Self::ConsultaDerivacao(erro) => Some(erro.subcodigo()),
        }
    }
}
mod escrita;

pub(crate) struct Pedido<'a> {
    pub(crate) anexado: &'a NumeroProcesso,
    pub(crate) principal: &'a NumeroProcesso,
    pub(crate) unidade: Option<&'a str>,
    pub(crate) confirmar: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ResultadoAnexacao {
    Previa,
    Confirmada,
}

pub(crate) struct Previa {
    pub(crate) resultado: ResultadoAnexacao,
    pub(crate) principal: String,
    pub(crate) id_principal: String,
    pub(crate) anexado: Destino,
    pub(crate) assinaturas_em_dia: bool,
    pub(crate) requisicoes: u8,
}

/// A URL das consultas tem origem exclusivamente no script servido; nunca
/// construímos URL assinada por inferência nem interpretamos o JavaScript.
pub(super) fn url_da_consulta(html: &str, inicio: &str) -> Result<String, &'static str> {
    let mut achados = html.split(inicio).skip(1);
    let primeiro = achados
        .next()
        .and_then(|resto| resto.split_once('\'').map(|(url, _)| url));
    if achados.next().is_some() {
        return Err("consultas_de_anexacao_repetidas");
    }
    let url = primeiro.ok_or("consulta_de_anexacao_nao_publicada")?;
    if !url.starts_with("controlador_ajax.php?") {
        return Err("consulta_de_anexacao_fora_da_forma_observada");
    }
    Ok(url.to_owned())
}

/// Um estado posterior é necessário para confirmar o efeito do único POST.
fn comprovar_na_arvore<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &Pedido<'_>,
    indice: &IndiceLocal,
    anexado: &Destino,
    anteriores: &[String],
    rede: &mut Contador<'_, V>,
) -> Result<(), FalhaAnexar> {
    let lido = obter_processo(
        sessao,
        &leitura(pedido.principal, pedido.unidade, false),
        indice,
        rede,
    )
    .map_err(|_| FalhaAnexar::EfeitoIncerto(EtapaNaoComprovada::ArvoreDoPrincipal))?;
    let depois = &lido.arvore.anexados;
    if !depois
        .iter()
        .any(|p| p.id == anexado.id && p.nup == anexado.nup)
        || !anteriores
            .iter()
            .all(|id| depois.iter().any(|p| &p.id == id))
    {
        return Err(FalhaAnexar::EfeitoIncerto(
            EtapaNaoComprovada::ArvoreDoPrincipal,
        ));
    }
    Ok(())
}

/// Sem --confirmar, a única escrita possível no transporte é a consulta AJAX
/// (POST de leitura, declarada e conferida). Nada de POST criador.
pub(crate) fn prever<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &Pedido<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<Previa, FalhaAnexar> {
    use FalhaAnexar as E;
    let instalacao = Instalacao::interpretar(sessao.origem)
        .ok_or(E::Tela(FalhaTransporte::OrigemNaoAutorizada))?;
    let mut rede = Contador::novo(envio);
    let (aberto, acoes) = abrir_para_escrever(
        sessao,
        &leitura(pedido.principal, pedido.unidade, true),
        indice,
        &mut rede,
    )
    .map_err(E::Abertura)?;
    let id = aberto.arvore.processo.id;
    let link = acoes.anexar.map_err(E::Barra)?.ok_or(E::LinkNaoPublicado)?;
    let cookie = ler_cookie(sessao)?;
    let (html, mime) =
        obter_tela(&mut rede, &cookie, link.as_str()).map_err(|erro| match erro {
            FalhaDaTela::Transporte(falha) => E::Tela(falha),
            FalhaDaTela::Redirecionada => E::TelaRedirecionada,
            FalhaDaTela::SessaoEncerrada => E::SessaoEncerrada,
        })?;
    let vista = ler_tela(&html).map_err(E::Estrutura)?;
    let codificacao = codificacao_da_tela(&mime)
        .ok_or(E::Estrutura("charset_da_tela_de_anexacao_nao_suportado"))?;
    let tela = TelaServida {
        url: link.as_str(),
        html: &html,
        codificacao,
    };
    let url = url_da_consulta(&html, "new infraAjaxComplementar('txtProtocolo','")
        .map_err(E::Estrutura)?;
    let consulta = derivar_consulta_ajax(
        &IDENTIFICAR,
        &instalacao,
        &tela,
        &url,
        &[("idProcedimento", pedido.anexado.as_str())],
    )
    .map_err(E::ConsultaDerivacao)?;
    let resposta = rede
        .post(consulta, &cookie)
        .map_err(E::ConsultaTransporte)?;
    if resposta.resposta.status_code == 302 {
        return Err(E::Estrutura("consulta_de_identidade_redirecionada"));
    }
    let xml = decodificar_resposta_xml(resposta.resposta).map_err(E::ConsultaResposta)?;
    let anexado = ler_destino(&xml).map_err(E::Estrutura)?;
    if anexado.nup != pedido.anexado.as_str() {
        return Err(E::Estrutura("nup_do_anexado_diferente_do_solicitado"));
    }
    decidir(pedido.principal.as_str(), &id, &anexado, &vista.anexados).map_err(E::Estrutura)?;

    let url = url_da_consulta(&html, "new infraAjaxComplementar(null,'").map_err(E::Estrutura)?;
    let consulta = derivar_consulta_ajax(
        &CONFERIR,
        &instalacao,
        &tela,
        &url,
        &[("idProcedimento", anexado.id.as_str())],
    )
    .map_err(E::ConsultaDerivacao)?;
    let resposta = rede
        .post(consulta, &cookie)
        .map_err(E::ConsultaTransporte)?;
    if resposta.resposta.status_code == 302 {
        return Err(E::Estrutura("consulta_de_assinaturas_redirecionada"));
    }
    let xml = decodificar_resposta_xml(resposta.resposta).map_err(E::ConsultaResposta)?;
    let assinaturas_em_dia = ler_confirmacao(&xml).map_err(E::Estrutura)?;
    let resultado = if pedido.confirmar {
        escrita::confirmar(
            sessao,
            &instalacao,
            &cookie,
            link.as_str(),
            &html,
            &mime,
            &id,
            &anexado,
            assinaturas_em_dia,
            &vista.anexados,
            &mut rede,
        )?;
        // O contrato exige a árvore da própria CLI, não somente o 302
        // nem a lista textual devolvida pelo formulário.
        comprovar_na_arvore(sessao, pedido, indice, &anexado, &vista.anexados, &mut rede)?;
        ResultadoAnexacao::Confirmada
    } else {
        ResultadoAnexacao::Previa
    };
    Ok(Previa {
        resultado,
        principal: pedido.principal.as_str().to_owned(),
        id_principal: id,
        anexado,
        assinaturas_em_dia,
        requisicoes: rede.feitas(),
    })
}

impl From<CookieIlegivel> for FalhaAnexar {
    fn from(erro: CookieIlegivel) -> Self {
        Self::Cookie(erro)
    }
}
