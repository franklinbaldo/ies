//! A sequência de `ies processo enviar` (contracts/cli/processo-enviar.md):
//! abertura, decisão antes da tela, a tela, a consulta do autocomplete, o
//! destino, um POST e a leitura de verificação, com a tabela de
//! reconciliação do contrato. Nenhum 302 é seguido; nenhuma escrita é
//! repetida.
use super::destino::{ler_resposta_da_consulta, resolver_destino};
use super::erro::{ErroEnviar as C, FalhaDaConsulta};
use super::tela::{
    AUTOCOMPLETAR_UNIDADE, ENVIAR_PROCESSO, TelaDeEnvio, campos_da_consulta_no_orgao,
    campos_do_envio, conferir_tela_de_envio, redirecionamento_esperado,
};
use super::{Destino, MetodoRemocao, PedidoDoEnvio, ProcessoEnviado};
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

/// Mesmo formulário usado para observar e, quando autorizado, limpar.
struct AnotacaoLida {
    presente: bool,
    html: String,
    codificacao: CodificacaoDaTela,
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
    let (aberto, acoes) = abrir_para_escrever(sessao, &leitura(true), indice, &mut rede)?;
    let unidade = aberto
        .unidade
        .clone()
        .ok_or(C::UnidadeDaSessaoNaoPublicada)?;
    let aberto_em = siglas_abertas(&aberto.arvore.aberto_em)
        .map_err(|motivo| C::SituacaoDoProcessoNaoLidaAntesDeEnviar { motivo })?;
    let link = conferir_link_do_envio(acoes.enviar, &aberto_em, &unidade)?;
    let id = aberto.arvore.processo.id.clone();
    let cookie = ler_cookie(sessao)
        .map_err(|falha| C::Formulario(ErroSubmissaoFormulario::Transporte(falha)))?;
    let tela = ler_tela(&mut rede, &cookie, link.as_str(), &id)?;
    let link_anotacao = if pedido.remover_anotacoes {
        acoes.anotacoes.map_err(C::LinkDasAnotacoes)?
    } else {
        None
    };
    let anotacao_lida = ler_anotacao_antes(&mut rede, &cookie, link_anotacao.as_deref(), &id)?;
    let anotacao_antes = anotacao_lida.as_ref().map(|lida| lida.presente);
    let metodo_remocao = escolher_metodo(
        pedido,
        &tela.conferida,
        anotacao_antes,
        link_anotacao.as_deref(),
        anotacao_lida.as_ref(),
    )?;
    let destino = consultar_destino(&mut rede, &cookie, &instalacao, &tela, pedido.para)?;
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
        remover_anotacoes: pedido.remover_anotacoes,
        anotacao_antes,
        anotacao_depois: None,
        metodo_remocao,
        destino,
        aberto_em,
        depois: None,
        reconciliado_por: None,
        requisicoes: rede.feitas(),
    };
    if !pedido.confirmar {
        return Ok(feito);
    }
    // BDD-333-03/04: grava anotação vazia somente se havia texto.
    // Cada POST é único e a releitura precisa comprovar a limpeza antes do envio.
    if metodo_remocao == Some(MetodoRemocao::LimparAnotacao)
        && let (Some(link), Some(lida)) = (link_anotacao.as_deref(), anotacao_lida.as_ref())
    {
        if lida.presente {
            limpar_anotacao(
                &mut rede,
                sessao,
                &instalacao,
                &cookie,
                link,
                lida,
                &feito.id_procedimento,
            )?;
        }
        feito.anotacao_depois = Some(false);
    }
    let campos = campos_do_envio(
        &tela.conferida,
        &feito.destino,
        pedido.manter_aberto,
        metodo_remocao == Some(MetodoRemocao::CheckboxNoEnvio),
    );
    let pares: Vec<(&str, &str)> = campos.iter().map(|(n, v)| (*n, v.as_str())).collect();
    let servida = TelaServida {
        url: &tela.url,
        html: &tela.html,
        codificacao: tela.codificacao,
    };
    let submissao = derivar_submissao(&ENVIAR_PROCESSO, &instalacao, &servida, "", &pares)?;
    let falha = enviar(&mut rede, sessao, submissao, &feito.id_procedimento)?;
    let verificacao = verificar(sessao, &leitura(false), indice, &mut rede);
    if metodo_remocao == Some(MetodoRemocao::CheckboxNoEnvio) {
        // Releitura independente da anotação mesmo se a árvore falhou.
        // A saída 3 pode assim distinguir remoção observada de tramitação
        // incerta, sem reenviar o POST nem seguir o redirect.
        let link = link_anotacao
            .as_deref()
            .ok_or(C::RemocaoDeAnotacoesSemVerificacao)?;
        feito.anotacao_depois = observar_anotacao_depois(
            &mut rede,
            &cookie,
            link,
            &feito.id_procedimento,
        );
    }
    let mut feito = conciliar(feito, falha, verificacao)?;
    if metodo_remocao == Some(MetodoRemocao::CheckboxNoEnvio) {
        match feito.anotacao_depois {
            Some(false) => {}
            Some(true) => return Err(C::AnotacoesNaoRemovidasAposEnvio),
            None => {
                return Err(C::EfeitoIncertoNaRemocaoDeAnotacoes {
                    depois: feito.depois,
                });
            }
        }
    }
    feito.requisicoes = rede.feitas();
    Ok(feito)
}

/// A barra só autoriza a escrita quando a situação lista a unidade da sessão.
fn conferir_link_do_envio(
    enviar: Result<
        Option<crate::arvore_processo::EnderecoDeEscrita>,
        crate::arvore_processo::ErroEnviarNaBarra,
    >,
    aberto_em: &[String],
    unidade: &str,
) -> Result<crate::arvore_processo::EnderecoDeEscrita, C> {
    let aberta = aberto_em.iter().any(|sigla| sigla == unidade);
    match (enviar, aberta) {
        (Ok(None) | Err(_), false) => Err(C::ProcessoNaoAbertoNaUnidadeDaSessao {
            aberto_em: aberto_em.to_vec(),
        }),
        (Ok(Some(_)), false) => Err(C::UnidadeDaSessaoAusenteDaSituacao {
            aberto_em: aberto_em.to_vec(),
        }),
        (Ok(None), true) => Err(C::EnvioNaoPublicadoNaBarra {
            aberto_em: aberto_em.to_vec(),
        }),
        (Err(erro), true) => Err(C::Barra(erro)),
        (Ok(Some(link)), true) => Ok(link),
    }
}

/// Decide a estratégia publicada na tela sem confundir leitura com gravação.
fn escolher_metodo(
    pedido: &PedidoDoEnvio<'_>,
    tela: &TelaDeEnvio,
    anotacao_antes: Option<bool>,
    link: Option<&str>,
    anotacao_lida: Option<&AnotacaoLida>,
) -> Result<Option<MetodoRemocao>, C> {
    if !pedido.remover_anotacoes {
        return Ok(None);
    }
    let metodo = if tela.remover_anotacoes_oferecida {
        MetodoRemocao::CheckboxNoEnvio
    } else if anotacao_antes.is_some() {
        if !anotacao_lida.is_some_and(|lida| super::anotacoes::salvar_oferecido(&lida.html)) {
            return Err(if pedido.confirmar {
                C::RemocaoDeAnotacoesSemVerificacao
            } else {
                C::RemocaoDeAnotacoesNaoOferecida
            });
        }
        MetodoRemocao::LimparAnotacao
    } else {
        return Err(C::RemocaoDeAnotacoesNaoOferecida);
    };
    if pedido.confirmar && link.is_none() {
        return Err(C::RemocaoDeAnotacoesSemVerificacao);
    }
    Ok(Some(metodo))
}

/// Prévia: o campo vazio só é ausência observada quando o formulário
/// pertence ao mesmo processo. Falta de link continua não observada.
fn ler_anotacao_antes<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    cookie: &str,
    link: Option<&str>,
    id: &str,
) -> Result<Option<AnotacaoLida>, C> {
    let Some(link) = link else {
        return Ok(None);
    };
    let (html, mime) = obter_tela(rede, cookie, link).map_err(|falha| match falha {
        FalhaDaTela::Transporte(falha) => C::TelaDasAnotacoes(falha),
        FalhaDaTela::Redirecionada => C::TelaDasAnotacoesRedirecionada,
        FalhaDaTela::SessaoEncerrada => C::SessaoEncerrada,
    })?;
    let presente =
        super::anotacoes::anotacao_presente(&html, id).map_err(C::LeituraDasAnotacoes)?;
    let codificacao = codificacao_da_tela(&mime).ok_or(C::CharsetDaTelaNaoSuportado)?;
    Ok(Some(AnotacaoLida {
        presente,
        html,
        codificacao,
    }))
}

/// Salva uma vez e comprova a limpeza antes de tramitar; efeito incerto
/// interrompe a operação, sem segunda gravação da anotação nem envio.
fn limpar_anotacao<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    sessao: &SessaoDoProcesso<'_>,
    instalacao: &Instalacao,
    cookie: &str,
    link: &str,
    lida: &AnotacaoLida,
    id: &str,
) -> Result<(), C> {
    let servida = TelaServida {
        url: link,
        html: &lida.html,
        codificacao: lida.codificacao,
    };
    let submissao = derivar_submissao(
        &super::anotacoes::REGISTRAR_ANOTACAO,
        instalacao,
        &servida,
        "",
        &[("txaDescricao", "")],
    )?;
    let mut saiu = false;
    let resposta = enviar_submissao_via(submissao, sessao.cookie, |s, c| {
        saiu = true;
        rede.post(s, c)
    });
    if !saiu {
        return Err(C::Formulario(
            resposta.expect_err("sem POST só falha na derivação"),
        ));
    }
    // Mesmo erro de transporte pós-POST não autoriza retry. A leitura decide.
    match conferir_anotacao_depois(rede, cookie, link, id, None) {
        Ok(()) => Ok(()),
        Err(C::AnotacoesNaoRemovidasAposEnvio) => Err(C::AnotacaoNaoLimpaAntesDeEnviar),
        Err(_) => Err(C::EfeitoIncertoNaLimpezaDaAnotacao),
    }
}

/// A árvore já confirmou a tramitação; outro GET decide se a anotação
/// realmente desapareceu, sem repetir POST diante de recusa ou incerteza.
fn conferir_anotacao_depois<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    cookie: &str,
    link: &str,
    id: &str,
    unidades_apos_envio: Option<&[String]>,
) -> Result<(), C> {
    match observar_anotacao_depois(rede, cookie, link, id) {
        Some(false) => Ok(()),
        Some(true) => Err(C::AnotacoesNaoRemovidasAposEnvio),
        None => Err(C::EfeitoIncertoNaRemocaoDeAnotacoes {
            depois: unidades_apos_envio.map(ToOwned::to_owned),
        }),
    }
}


/// A remoção é um efeito distinto da tramitação: observar separadamente
/// permite registrar o que ocorreu mesmo se a árvore ficou indisponível.
fn observar_anotacao_depois<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    cookie: &str,
    link: &str,
    id: &str,
) -> Option<bool> {
    obter_tela(rede, cookie, link)
        .ok()
        .and_then(|(html, _)| super::anotacoes::anotacao_presente(&html, id).ok())
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
    let conferida = conferir_tela_de_envio(&html, id)?;
    let codificacao = codificacao_da_tela(&mime).ok_or(C::CharsetDaTelaNaoSuportado)?;
    Ok(TelaLida {
        url: url.to_owned(),
        html,
        codificacao,
        conferida,
    })
}

/// Os POST da consulta do autocomplete (leitura) que a busca planeja e a
/// escolha do destino.
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
    let conferida = &tela.conferida;
    resolver_destino(para, &conferida.orgao, &conferida.orgaos, |termo, orgao| {
        let campos = campos_da_consulta_no_orgao(conferida, termo, orgao);
        let consulta = derivar_consulta_ajax(
            &AUTOCOMPLETAR_UNIDADE,
            instalacao,
            &servida,
            &conferida.autocompletar,
            &campos,
        )
        .map_err(|erro| C::Consulta(FalhaDaConsulta::Derivacao(erro)))?;
        let resposta = rede
            .post(consulta, cookie)
            .map_err(|falha| C::Consulta(FalhaDaConsulta::Transporte(falha)))?;
        ler_resposta_da_consulta(resposta).map_err(C::Consulta)
    })
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
    let resposta = enviar_submissao_via(submissao, sessao.cookie, |s, c| {
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
            anotacao_depois: feito.anotacao_depois,
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
