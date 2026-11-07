//! Prévia Web de relacionamento de processos; não faz a escrita.
mod tela;
#[cfg(test)]
#[path = "processo_relacionar/tests.rs"]
mod tests;
#[cfg(test)]
#[path = "processo_relacionar/tests_fluxo.rs"]
mod tests_fluxo;

use crate::arvore_processo::ErroRelacionarNaBarra;
use crate::formulario_web::{
    ConsultaAjax, EfeitoAvaliado, ErroSubmissaoFormulario, TelaServida, derivar_consulta_ajax,
};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::{Contador, FalhaDaTela, leitura, ler_cookie, obter_tela};
use crate::processo_obter::{
    EnvioDoProcesso, ErroProcessoObter, SessaoDoProcesso, abrir_para_escrever,
};
use crate::transporte::{FalhaTransporte, decodificar_resposta};
use crate::visao_do_controle::EnvioDoControle as _;
use tela::{DestinoConsultado, Relacionamento, ler_destino, ler_linhas, url_complementar};

const CONSULTA: ConsultaAjax = ConsultaAjax {
    acao_ajax: "protocolo_RI1023",
    campos: &["idProcedimento"],
    efeito: EfeitoAvaliado::ConsultaSemEfeitoObservado {
        evidencia: "tests/paginas_reais/procedimento_relacionar.html no Todos @323eb01: consulta do botão Pesquisar",
    },
};

#[derive(Debug)]
pub(crate) enum FalhaRelacionar {
    ConfirmacaoAindaNaoValidada,
    Abertura(ErroProcessoObter),
    Barra(ErroRelacionarNaBarra),
    LinkNaoPublicado,
    Tela(FalhaTransporte),
    TelaRedirecionada,
    SessaoEncerrada,
    Estrutura(&'static str),
    ConsultaDerivacao(ErroSubmissaoFormulario),
    ConsultaTransporte(FalhaTransporte),
    ConsultaResposta(FalhaTransporte),
    RelacionamentoConsigoMesmo,
    RemocaoNaoOferecida,
}
impl FalhaRelacionar {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::ConfirmacaoAindaNaoValidada => "confirmacao_de_relacionamento_nao_validada",
            Self::Abertura(erro) => erro.subcodigo(),
            Self::Barra(erro) => erro.subcodigo(),
            Self::LinkNaoPublicado => "relacionar_nao_publicado_na_barra",
            Self::Tela(erro) | Self::ConsultaTransporte(erro) | Self::ConsultaResposta(erro) => {
                erro.subcodigo()
            }
            Self::TelaRedirecionada => "tela_de_relacionamentos_redirecionada",
            Self::SessaoEncerrada => "sessao_encerrada_rode_ies_login",
            Self::Estrutura(subcodigo) => subcodigo,
            Self::ConsultaDerivacao(erro) => erro.subcodigo(),
            Self::RelacionamentoConsigoMesmo => "relacionamento_consigo_mesmo",
            Self::RemocaoNaoOferecida => "remocao_de_relacionamento_nao_oferecida",
        }
    }
}
pub(crate) struct Pedido<'a> {
    pub(crate) nup: &'a NumeroProcesso,
    pub(crate) com: &'a NumeroProcesso,
    pub(crate) unidade: Option<&'a str>,
    pub(crate) remover: bool,
    pub(crate) confirmar: bool,
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Previa {
    pub(crate) nup: String,
    pub(crate) id_procedimento: String,
    pub(crate) destino: String,
    pub(crate) id_destino: Option<String>,
    pub(crate) remover: bool,
    pub(crate) sem_efeito: bool,
    pub(crate) requisicoes: u8,
}

/// A confirmação é recusada ANTES da sessão e da primeira requisição.
/// Não supor que a prévia prova o efeito da escrita.
pub(crate) fn prever<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &Pedido<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<Previa, FalhaRelacionar> {
    use FalhaRelacionar as E;
    if pedido.confirmar {
        return Err(E::ConfirmacaoAindaNaoValidada);
    }
    let instalacao = Instalacao::interpretar(sessao.origem)
        .ok_or(E::Tela(FalhaTransporte::OrigemNaoAutorizada))?;
    let mut rede = Contador::novo(envio);
    let (aberto, acoes) = abrir_para_escrever(
        sessao,
        &leitura(pedido.nup, pedido.unidade, true),
        indice,
        &mut rede,
    )
    .map_err(E::Abertura)?;
    let id = aberto.arvore.processo.id;
    let link = acoes
        .relacionar
        .map_err(E::Barra)?
        .ok_or(E::LinkNaoPublicado)?;
    let cookie = ler_cookie(sessao).map_err(E::Tela)?;
    let (html, mime) = obter_tela(&mut rede, &cookie, link.as_str()).map_err(|e| match e {
        FalhaDaTela::Transporte(falha) => E::Tela(falha),
        FalhaDaTela::Redirecionada => E::TelaRedirecionada,
        FalhaDaTela::SessaoEncerrada => E::SessaoEncerrada,
    })?;
    let relacionados = ler_linhas(&html).map_err(E::Estrutura)?;
    let mut previa = Previa {
        nup: pedido.nup.as_str().to_owned(),
        id_procedimento: id.clone(),
        destino: pedido.com.as_str().to_owned(),
        id_destino: None,
        remover: pedido.remover,
        sem_efeito: false,
        requisicoes: rede.feitas(),
    };
    if pedido.remover {
        let achados: Vec<&Relacionamento> = relacionados
            .iter()
            .filter(|linha| linha.nup == pedido.com.as_str())
            .collect();
        match achados.as_slice() {
            [] => previa.sem_efeito = true,
            [linha] => {
                if !linha.pode_remover {
                    return Err(E::RemocaoNaoOferecida);
                }
                previa.destino.clone_from(&linha.nup);
                previa.id_destino = Some(linha.id.clone());
            }
            _ => return Err(E::Estrutura("relacionamentos_repetidos_na_tela")),
        }
        return Ok(previa);
    }
    // A tabela já relaciona NUP e id do destino: consultar o complementar
    // outra vez não acrescenta evidência e faz um POST de leitura dispensável.
    let existentes: Vec<_> = relacionados
        .iter()
        .filter(|linha| linha.nup == pedido.com.as_str())
        .collect();
    match existentes.as_slice() {
        [] => {}
        [linha] => {
            if linha.id == id {
                return Err(E::RelacionamentoConsigoMesmo);
            }
            previa.id_destino = Some(linha.id.clone());
            previa.destino.clone_from(&linha.nup);
            previa.sem_efeito = true;
            return Ok(previa);
        }
        _ => return Err(E::Estrutura("relacionamentos_repetidos_na_tela")),
    }
    // O AJAX de consulta é um POST de LEITURA, não uma gravação da relação.
    let codificacao = codificacao_da_tela(&mime).ok_or(E::Estrutura(
        "charset_da_tela_de_relacionamentos_nao_suportado",
    ))?;
    let tela = TelaServida {
        url: link.as_str(),
        html: &html,
        codificacao,
    };
    let url = url_complementar(&html).map_err(E::Estrutura)?;
    let consulta = derivar_consulta_ajax(
        &CONSULTA,
        &instalacao,
        &tela,
        &url,
        &[("idProcedimento", pedido.com.as_str())],
    )
    .map_err(E::ConsultaDerivacao)?;
    let resposta = rede
        .post(consulta, &cookie)
        .map_err(E::ConsultaTransporte)?;
    if resposta.resposta.status_code == 302 {
        return Err(E::Estrutura("consulta_de_relacionamento_redirecionada"));
    }
    let xml = decodificar_resposta(resposta.resposta).map_err(E::ConsultaResposta)?;
    let DestinoConsultado {
        id: destino_id,
        nup,
    } = ler_destino(&xml).map_err(E::Estrutura)?;
    if destino_id == id {
        return Err(E::RelacionamentoConsigoMesmo);
    }
    previa.sem_efeito = relacionados.iter().any(|linha| linha.id == destino_id);
    previa.id_destino = Some(destino_id);
    previa.destino = nup;
    previa.requisicoes = rede.feitas();
    Ok(previa)
}
