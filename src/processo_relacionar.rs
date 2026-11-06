//! Prévia Web de relacionamento de processos; não faz a escrita.
mod tela;
#[cfg(test)]
#[path = "processo_relacionar/tests.rs"]
mod tests;
#[cfg(test)]
#[path = "processo_relacionar/tests_escrita.rs"]
mod tests_escrita;

use crate::arvore_processo::ErroRelacionarNaBarra;
use crate::formulario_web::{
    ConferenciaDaUnidade, ConsultaAjax, EfeitoAvaliado, EnvioDoFormulario, ErroSubmissaoFormulario,
    EstadoDoProcesso, OperacaoDeFormulario, ResultadoDoEnvio, TelaServida, derivar_consulta_ajax,
    derivar_submissao, derivar_submissao_com_action, enviar_submissao_via,
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

const ADICIONAR_RELACIONAMENTO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "procedimento_relacionar",
    formulario: "frmProcedimentoListarRelacionamentos",
    envio: EnvioDoFormulario::BotaoDeEnvio("sbmAdicionar"),
    campos_ocultos_exigidos: &["hdnIdProtocolo"],
    campos_ocultos_reescritos: &["hdnIdProtocolo"],
    campos_usuario: &["txtProtocolo", "txtIdentificacaoProtocolo"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::Relacionamento,
    },
};

const EXCLUIR_RELACIONAMENTO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "procedimento_excluir_relacionamento",
    formulario: "frmProcedimentoListarRelacionamentos",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &["hdnInfraItemId"],
    campos_ocultos_reescritos: &["hdnInfraItemId"],
    campos_usuario: &[],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::Relacionamento,
    },
};

#[derive(Debug)]
pub(crate) enum FalhaRelacionar {
    #[allow(dead_code)]
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
    EfeitoIncerto,
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
            Self::EfeitoIncerto => "efeito_incerto",
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
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Previa {
    pub(crate) nup: String,
    pub(crate) id_procedimento: String,
    pub(crate) destino: String,
    pub(crate) id_destino: Option<String>,
    pub(crate) remover: bool,
    pub(crate) confirmado: bool,
    pub(crate) alterado: bool,
    pub(crate) sem_efeito: bool,
    pub(crate) requisicoes: u8,
}

#[allow(clippy::too_many_lines)] // fluxo sequencial até a verificação, sem retries.
pub(crate) fn prever<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &Pedido<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<Previa, FalhaRelacionar> {
    use FalhaRelacionar as E;
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
        confirmado: false,
        alterado: false,
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
        if pedido.confirmar && !previa.sem_efeito {
            let id_alvo = previa
                .id_destino
                .as_deref()
                .ok_or(E::Estrutura("destino_a_remover_sem_identificador"))?;
            executar_remocao(
                sessao,
                &instalacao,
                &cookie,
                link.as_str(),
                &html,
                &mime,
                id_alvo,
                &mut rede,
            )?;
            previa.confirmado = true;
            previa.alterado = true;
        }
        previa.requisicoes = rede.feitas();
        return Ok(previa);
    }
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
    previa.id_destino = Some(destino_id.clone());
    previa.destino = nup;
    if pedido.confirmar && !previa.sem_efeito {
        executar_adicao(
            sessao,
            &instalacao,
            &cookie,
            link.as_str(),
            &tela,
            destino_id.as_str(),
            previa.destino.as_str(),
            &mut rede,
        )?;
        previa.confirmado = true;
        previa.alterado = true;
    }
    previa.requisicoes = rede.feitas();
    Ok(previa)
}

#[allow(clippy::too_many_arguments)]
fn executar_remocao<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    instalacao: &Instalacao,
    cookie: &str,
    link: &str,
    html: &str,
    mime: &str,
    id_alvo: &str,
    rede: &mut Contador<'_, V>,
) -> Result<(), FalhaRelacionar> {
    use FalhaRelacionar as E;
    let codificacao = codificacao_da_tela(mime).ok_or(E::Estrutura(
        "charset_da_tela_de_relacionamentos_nao_suportado",
    ))?;
    let tela = TelaServida {
        url: link,
        html,
        codificacao,
    };
    let url_exclusao = tela::url_exclusao(html).map_err(E::Estrutura)?;
    let submissao = derivar_submissao_com_action(
        &EXCLUIR_RELACIONAMENTO,
        instalacao,
        &tela,
        "",
        &[("hdnInfraItemId", id_alvo)],
        Some(&url_exclusao),
    )
    .map_err(E::ConsultaDerivacao)?;
    let mut enviado = false;
    let _ = enviar_submissao_via(submissao, sessao.cookie, |s, c| {
        enviado = true;
        rede.post(s, c)
    });
    if !enviado {
        return Err(E::Estrutura("falha_antes_do_envio_do_formulario"));
    }
    let (re_html, _) = obter_tela(rede, cookie, link).map_err(|_| E::EfeitoIncerto)?;
    let novas_linhas = ler_linhas(&re_html).map_err(|_| E::EfeitoIncerto)?;
    if novas_linhas.iter().any(|linha| linha.id == id_alvo) {
        return Err(E::EfeitoIncerto);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn executar_adicao<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    instalacao: &Instalacao,
    cookie: &str,
    link: &str,
    tela: &TelaServida<'_>,
    destino_id: &str,
    destino_nup: &str,
    rede: &mut Contador<'_, V>,
) -> Result<(), FalhaRelacionar> {
    use FalhaRelacionar as E;
    let submissao = derivar_submissao(
        &ADICIONAR_RELACIONAMENTO,
        instalacao,
        tela,
        "",
        &[
            ("hdnIdProtocolo", destino_id),
            ("txtProtocolo", destino_nup),
            ("txtIdentificacaoProtocolo", ""),
        ],
    )
    .map_err(E::ConsultaDerivacao)?;
    let mut enviado = false;
    let _ = enviar_submissao_via(submissao, sessao.cookie, |s, c| {
        enviado = true;
        rede.post(s, c)
    });
    if !enviado {
        return Err(E::Estrutura("falha_antes_do_envio_do_formulario"));
    }
    let (re_html, _) = obter_tela(rede, cookie, link).map_err(|_| E::EfeitoIncerto)?;
    let novas_linhas = ler_linhas(&re_html).map_err(|_| E::EfeitoIncerto)?;
    if !novas_linhas.iter().any(|linha| linha.id == destino_id) {
        return Err(E::EfeitoIncerto);
    }
    Ok(())
}
