//! Prévia Web de anexação: abre principal, lê a tela e consulta somente
//! a identidade do anexado e sua condição de assinatura. Não grava nada.
//! POST de escrita apenas após prévia e confirmação, com releitura de prova.
use super::{
    Destino, ErroAnexacaoRecusada, ErroComplementosDaAnexacao, ErroTelaDeAnexacao, decidir,
    ler_confirmacao, ler_destino, ler_tela,
};
use crate::arvore_processo::ErroAnexarNaBarra;
use crate::formulario_web::{
    ConsultaAjax, EfeitoAvaliado, ErroSubmissaoFormulario, TelaServida, derivar_consulta_ajax,
};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::{Contador, FalhaDaTela, leitura, obter_tela};
use crate::processo_obter::{
    EnvioDoProcesso, ErroProcessoObter, SessaoDoProcesso, abrir_para_escrever, obter_processo,
};
use crate::transporte::{FalhaTransporte, decodificar_resposta_xml};
use crate::visao_do_controle::EnvioDoControle as _;

const IDENTIFICAR: ConsultaAjax = ConsultaAjax {
    acao_ajax: crate::acao_do_sei::AcaoAjaxDoSei::ProtocoloRi1023,
    campos: &["idProcedimento"],
    efeito: EfeitoAvaliado::ConsultaSemEfeitoObservado {
        evidencia: "Todos/src/todos/anexacao.py @323eb01, UnB 4.0.9",
    },
};
const CONFERIR: ConsultaAjax = ConsultaAjax {
    acao_ajax: crate::acao_do_sei::AcaoAjaxDoSei::ProcedimentoAnexacaoVerificar,
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
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum FalhaAnexar {
    /// O processo a anexar é o próprio principal; recusado antes da sessão.
    AnexacaoAoProprioProcesso,
    NaoConfirmado(EtapaNaoComprovada),
    /// Um POST de anexação foi tentado, mas a resposta falhou sem prova do efeito.
    NaoConfirmadoNoPost(#[source] ErroSubmissaoFormulario),
    /// A releitura da árvore do principal falhou após o POST.
    NaoConfirmadoNaArvoreDoPrincipal(#[source] ErroProcessoObter),
    /// A releitura da lista após o POST falhou no transporte.
    NaoConfirmadoNaListaAposPost(#[source] FalhaDaTela),
    /// A lista relida após o POST não tem a forma observada.
    NaoConfirmadoNaTelaDaListaAposPost(#[source] ErroTelaDeAnexacao),
    Abertura(ErroProcessoObter),
    Barra(ErroAnexarNaBarra),
    LinkNaoPublicado,
    Tela(FalhaTransporte),
    TelaRedirecionada,
    SessaoEncerrada,
    TelaDeAnexacao(#[from] ErroTelaDeAnexacao),
    CharsetDaTelaDeAnexacaoNaoSuportado,
    Consulta(#[from] ErroConsultaDeAnexacao),
    ConsultaDeIdentidadeRedirecionada,
    ConsultaDeAssinaturasRedirecionada,
    Complementos(#[from] ErroComplementosDaAnexacao),
    NupDoAnexadoDiferenteDoSolicitado,
    Recusada(#[from] ErroAnexacaoRecusada),
    ConsultaDerivacao(ErroSubmissaoFormulario),
    ConsultaTransporte(FalhaTransporte),
    ConsultaResposta(FalhaTransporte),
}
pub(super) mod escrita;

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
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroConsultaDeAnexacao {
    ConsultasDeAnexacaoRepetidas,
    ConsultaDeAnexacaoNaoPublicada,
    ConsultaDeAnexacaoForaDaFormaObservada,
}

pub(super) fn url_da_consulta(html: &str, inicio: &str) -> Result<String, ErroConsultaDeAnexacao> {
    let mut achados = html.split(inicio).skip(1);
    let primeiro = achados
        .next()
        .and_then(|resto| resto.split_once('\'').map(|(url, _)| url));
    if achados.next().is_some() {
        return Err(ErroConsultaDeAnexacao::ConsultasDeAnexacaoRepetidas);
    }
    let url = primeiro.ok_or(ErroConsultaDeAnexacao::ConsultaDeAnexacaoNaoPublicada)?;
    if !url.starts_with("controlador_ajax.php?") {
        return Err(ErroConsultaDeAnexacao::ConsultaDeAnexacaoForaDaFormaObservada);
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
    .map_err(FalhaAnexar::NaoConfirmadoNaArvoreDoPrincipal)?;
    let depois = &lido.arvore.anexados;
    if !depois
        .iter()
        .any(|p| p.id == anexado.id && p.nup == anexado.nup)
        || !anteriores
            .iter()
            .all(|id| depois.iter().any(|p| &p.id == id))
    {
        return Err(FalhaAnexar::NaoConfirmado(
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
    let instalacao = Instalacao::interpretar(sessao.origem)
        .ok_or(FalhaAnexar::Tela(FalhaTransporte::OrigemNaoAutorizada))?;
    let mut rede = Contador::novo(envio);
    let (aberto, acoes) = abrir_para_escrever(
        sessao,
        &leitura(pedido.principal, pedido.unidade, true),
        indice,
        &mut rede,
    )
    .map_err(FalhaAnexar::Abertura)?;
    let id = aberto.arvore.processo.id;
    let link = acoes
        .anexar
        .map_err(FalhaAnexar::Barra)?
        .ok_or(FalhaAnexar::LinkNaoPublicado)?;
    let cookie = acoes.cookie.clone();
    let (html, mime) =
        obter_tela(&mut rede, &cookie, link.as_str()).map_err(|erro| match erro {
            FalhaDaTela::Transporte(falha) => FalhaAnexar::Tela(falha),
            FalhaDaTela::Redirecionada => FalhaAnexar::TelaRedirecionada,
            FalhaDaTela::SessaoEncerrada(crate::erro_publico::SessaoEncerrada) => {
                FalhaAnexar::SessaoEncerrada
            }
        })?;
    let vista = ler_tela(&html)?;
    let codificacao =
        codificacao_da_tela(&mime).ok_or(FalhaAnexar::CharsetDaTelaDeAnexacaoNaoSuportado)?;
    let tela = TelaServida {
        url: link.as_str(),
        html: &html,
        codificacao,
    };
    let consulta = ConsultaDaTela {
        instalacao: &instalacao,
        tela: &tela,
        cookie: &cookie,
    };
    let xml = consulta.enviar(
        &mut rede,
        &IDENTIFICAR,
        "new infraAjaxComplementar('txtProtocolo','",
        pedido.anexado.as_str(),
        FalhaAnexar::ConsultaDeIdentidadeRedirecionada,
    )?;
    let anexado = ler_destino(&xml)?;
    if anexado.nup != pedido.anexado.as_str() {
        return Err(FalhaAnexar::NupDoAnexadoDiferenteDoSolicitado);
    }
    decidir(pedido.principal.as_str(), &id, &anexado, &vista.anexados)?;

    let xml = consulta.enviar(
        &mut rede,
        &CONFERIR,
        "new infraAjaxComplementar(null,'",
        anexado.id.as_str(),
        FalhaAnexar::ConsultaDeAssinaturasRedirecionada,
    )?;
    let assinaturas_em_dia = ler_confirmacao(&xml)?;
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

/// Uma consulta AJAX sem efeito declarada na tela de anexação: um POST, sem
/// seguir 302, e o XML devolvido.
struct ConsultaDaTela<'a> {
    instalacao: &'a Instalacao,
    tela: &'a TelaServida<'a>,
    cookie: &'a str,
}

impl ConsultaDaTela<'_> {
    fn enviar<V: EnvioDoProcesso>(
        &self,
        rede: &mut Contador<'_, V>,
        operacao: &ConsultaAjax,
        declaracao: &str,
        id_procedimento: &str,
        redirecionada: FalhaAnexar,
    ) -> Result<crate::tela_html::HtmlServido, FalhaAnexar> {
        let url = url_da_consulta(self.tela.html, declaracao)?;
        let consulta = derivar_consulta_ajax(
            operacao,
            self.instalacao,
            self.tela,
            &url,
            &[("idProcedimento", id_procedimento)],
        )
        .map_err(FalhaAnexar::ConsultaDerivacao)?;
        let resposta = rede
            .post(consulta, self.cookie)
            .map_err(FalhaAnexar::ConsultaTransporte)?;
        if resposta.resposta.status_code == 302 {
            return Err(redirecionada);
        }
        decodificar_resposta_xml(resposta.resposta).map_err(FalhaAnexar::ConsultaResposta)
    }
}
