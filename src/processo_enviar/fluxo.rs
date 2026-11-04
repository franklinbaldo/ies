//! A sequência de `ies processo enviar` (contracts/cli/processo-enviar.md):
//! abertura, decisão antes da tela, a tela, a consulta do autocomplete, o
//! destino, um POST e a leitura de verificação, com a tabela de
//! reconciliação do contrato. Nenhum 302 é seguido; nenhuma escrita é
//! repetida.
use super::destino::{escolher_destino, ler_resposta_da_consulta};
use super::erro::{ErroEnviar as C, FalhaDaConsulta};
use super::tela::{
    AUTOCOMPLETAR_UNIDADE, ENVIAR_PROCESSO, TelaDeEnvio, campos_da_consulta, campos_do_envio,
    conferir_tela_de_envio, redirecionamento_esperado,
};
use super::{Destino, PedidoDoEnvio, ProcessoEnviado};
use crate::formulario_web::{
    CodificacaoDaTela, ErroSubmissaoFormulario, RespostaDoEnvio, SubmissaoDerivada, TelaServida,
    derivar_consulta_ajax, derivar_submissao, enviar_submissao_via,
};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo_concluir::escrita::{
    Contador, FalhaDaTela, leitura, ler_cookie, obter_tela, siglas_abertas, verificar,
};
use crate::processo_concluir::{FalhaDoEnvio, VerificacaoPorLeitura};
use crate::processo_obter::{
    EnvioDoProcesso, ErroProcessoObter, SessaoDoProcesso, abrir_para_escrever,
};
use crate::transporte::FalhaTransporte;
use crate::visao_do_controle::EnvioDoControle as _;

/// A tela servida e conferida: HTML, URL, charset e o que ela publica.
struct TelaLida {
    url: String,
    html: String,
    codificacao: CodificacaoDaTela,
    conferida: TelaDeEnvio,
}

/// Sem POST de escrita na prévia; um POST, nunca repetido.
pub(crate) fn enviar_processo<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDoEnvio<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<ProcessoEnviado, C> {
    let instalacao = Instalacao::interpretar(sessao.origem).ok_or(C::Abertura(
        ErroProcessoObter::Transporte(FalhaTransporte::OrigemNaoAutorizada),
    ))?;
    let mut rede = Contador::novo(envio);
    let leitura = |unidade_exigida| leitura(pedido.nup, pedido.unidade, unidade_exigida);
    let (aberto, acoes) =
        abrir_para_escrever(sessao, &leitura(true), indice, &mut rede).map_err(C::Abertura)?;
    let unidade = aberto
        .unidade
        .clone()
        .ok_or(C::UnidadeDaSessaoNaoPublicada)?;
    let aberto_em = siglas_abertas(&aberto.arvore.aberto_em)
        .map_err(|motivo| C::SituacaoDoProcessoNaoLidaAntesDeEnviar { motivo })?;
    let link = match (acoes.enviar, aberto_em.contains(&unidade)) {
        // Fora da unidade, a barra (lida ou não) não muda a recusa.
        (Ok(None) | Err(_), false) => {
            return Err(C::ProcessoNaoAbertoNaUnidadeDaSessao { aberto_em });
        }
        (Ok(Some(_)), false) => return Err(C::UnidadeDaSessaoAusenteDaSituacao { aberto_em }),
        (Ok(None), true) => return Err(C::EnvioNaoPublicadoNaBarra { aberto_em }),
        (Err(erro), true) => return Err(C::Barra(erro)),
        (Ok(Some(link)), true) => link,
    };
    let id = aberto.arvore.processo.id.clone();
    let cookie = ler_cookie(sessao)
        .map_err(|falha| C::Formulario(ErroSubmissaoFormulario::Transporte(falha)))?;
    let tela = ler_tela(&mut rede, &cookie, link.as_str(), &id)?;
    let destino = consultar_destino(&mut rede, &cookie, &instalacao, &tela, pedido.para.trim())?;
    if aberto_em.contains(&destino.sigla) {
        return Err(C::ProcessoJaAbertoNoDestino { aberto_em });
    }
    let mut feito = ProcessoEnviado {
        nup: pedido.nup.as_str().to_owned(),
        id_procedimento: id,
        unidade,
        confirmado: pedido.confirmar,
        alterado: false,
        manter_aberto: pedido.manter_aberto,
        destino,
        aberto_em,
        depois: None,
        reconciliado_por: None,
        requisicoes: rede.feitas(),
    };
    if !pedido.confirmar {
        return Ok(feito);
    }
    let campos = campos_do_envio(&tela.conferida, &feito.destino, pedido.manter_aberto);
    let pares: Vec<(&str, &str)> = campos.iter().map(|(n, v)| (*n, v.as_str())).collect();
    let servida = TelaServida {
        url: &tela.url,
        html: &tela.html,
        codificacao: tela.codificacao,
    };
    let submissao = derivar_submissao(&ENVIAR_PROCESSO, &instalacao, &servida, "", &pares)
        .map_err(C::Formulario)?;
    let falha = enviar(&mut rede, sessao, submissao, &feito.id_procedimento)?;
    let verificacao = verificar(sessao, &leitura(false), indice, &mut rede);
    feito.requisicoes = rede.feitas();
    conciliar(feito, falha, verificacao)
}

/// GET da tela de envio (o link da barra), conferida sem rede.
fn ler_tela<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    cookie: &str,
    url: &str,
    id: &str,
) -> Result<TelaLida, C> {
    let (html, mime) = obter_tela(rede, cookie, url).map_err(|falha| match falha {
        FalhaDaTela::Transporte(falha) => C::TelaDaEscrita(falha),
        FalhaDaTela::Redirecionada => C::TelaDaEscritaRedirecionada,
        FalhaDaTela::SessaoEncerrada => C::SessaoEncerrada,
    })?;
    let conferida = conferir_tela_de_envio(&html, id).map_err(C::TelaDeEnvio)?;
    let codificacao = codificacao_da_tela(&mime).ok_or(C::CharsetDaTelaNaoSuportado)?;
    Ok(TelaLida {
        url: url.to_owned(),
        html,
        codificacao,
        conferida,
    })
}

/// O POST da consulta do autocomplete (leitura) e a escolha do destino.
fn consultar_destino<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    cookie: &str,
    instalacao: &Instalacao,
    tela: &TelaLida,
    para: &str,
) -> Result<Destino, C> {
    let servida = TelaServida {
        url: &tela.url,
        html: &tela.html,
        codificacao: tela.codificacao,
    };
    let campos = campos_da_consulta(&tela.conferida, para);
    let consulta = derivar_consulta_ajax(
        &AUTOCOMPLETAR_UNIDADE,
        instalacao,
        &servida,
        &tela.conferida.autocompletar,
        &campos,
    )
    .map_err(|erro| C::Consulta(FalhaDaConsulta::Derivacao(erro)))?;
    let resposta = rede
        .post(consulta, cookie)
        .map_err(|falha| C::Consulta(FalhaDaConsulta::Transporte(falha)))?;
    let itens = ler_resposta_da_consulta(resposta).map_err(C::Consulta)?;
    escolher_destino(para, &itens)
}

/// Um único POST. `Ok(None)`: o 302 esperado; `Ok(Some(falha))`: o pedido
/// saiu e a resposta não é a esperada; `Err`: nada saiu.
fn enviar<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    sessao: &SessaoDoProcesso<'_>,
    submissao: SubmissaoDerivada,
    id: &str,
) -> Result<Option<FalhaDoEnvio>, C> {
    let mut enviado = false;
    let resposta = enviar_submissao_via(submissao, sessao.cookie_file, |s, c| {
        enviado = true;
        rede.post(s, c)
    });
    match resposta {
        Ok(RespostaDoEnvio::Redirecionamento(r)) => {
            Ok(
                (!redirecionamento_esperado(r.localizacao(), r.url_do_envio(), id))
                    .then_some(FalhaDoEnvio::RedirecionamentoNaoReconhecidoAposEnviar),
            )
        }
        Ok(RespostaDoEnvio::Html(_)) => Ok(Some(FalhaDoEnvio::RespostaDoEnvioSemRedirecionamento)),
        Err(erro) if enviado => Ok(Some(FalhaDoEnvio::Envio(erro))),
        Err(erro) => Err(C::Formulario(erro)),
    }
}

/// Tabela de reconciliação do contrato.
fn conciliar(
    mut feito: ProcessoEnviado,
    falha: Option<FalhaDoEnvio>,
    verificacao: VerificacaoPorLeitura,
) -> Result<ProcessoEnviado, C> {
    let VerificacaoPorLeitura::Lida(depois) = verificacao else {
        return Err(C::EfeitoIncertoNoEnvio {
            aberto_em: feito.aberto_em,
            falha_do_envio: falha,
            verificacao: Box::new(verificacao),
        });
    };
    if !depois.contains(&feito.destino.sigla) {
        return Err(C::EnvioNaoAplicado {
            aberto_em: feito.aberto_em,
            depois,
            falha_do_envio: falha,
        });
    }
    if depois.contains(&feito.unidade) != feito.manter_aberto {
        return Err(C::SituacaoDaOrigemDiferenteDaPedidaAposEnvio {
            aberto_em: feito.aberto_em,
            depois,
            falha_do_envio: falha,
        });
    }
    feito.alterado = true;
    feito.depois = Some(depois);
    feito.reconciliado_por = falha;
    Ok(feito)
}
