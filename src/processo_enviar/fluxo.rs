//! A sequência de `ies processo enviar` (contracts/cli/processo-enviar.md):
//! abertura, decisão antes da tela, a tela, a consulta do autocomplete, o
//! destino, um POST e a leitura de verificação, com a tabela de
//! reconciliação do contrato. Nenhum 302 é seguido; nenhuma escrita é
//! repetida.
use super::destino::{ler_resposta_da_consulta, resolver_destino};
use super::erro::{ErroEnviar, FalhaDaConsulta};
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
) -> Result<ProcessoEnviado, ErroEnviar> {
    let instalacao = Instalacao::interpretar(sessao.origem).ok_or(ErroEnviar::Abertura(
        ErroProcessoObter::Transporte(FalhaTransporte::OrigemNaoAutorizada),
    ))?;
    let mut rede = Contador::novo(envio);
    let leitura = |unidade_exigida| leitura(pedido.nup, pedido.unidade, unidade_exigida);
    let (aberto, acoes) = abrir_para_escrever(sessao, &leitura(true), indice, &mut rede)?;
    let unidade = aberto
        .unidade
        .clone()
        .ok_or(ErroEnviar::UnidadeDaSessaoNaoPublicada)?;
    let aberto_em = siglas_abertas(&aberto.arvore.aberto_em)
        .map_err(|motivo| ErroEnviar::SituacaoDoProcessoNaoLidaAntesDeEnviar { motivo })?;
    let link = conferir_link_do_envio(acoes.enviar, &aberto_em, &unidade)?;
    let id = aberto.arvore.processo.id.clone();
    let cookie = ler_cookie(sessao)?;
    let tela = ler_tela(&mut rede, &cookie, link.as_str(), &id)?;
    let link_anotacao = if pedido.remover_anotacoes {
        acoes
            .anotacoes
            .map_err(super::anotacoes::ErroDasAnotacoes::LinkDasAnotacoes)?
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
        return Err(ErroEnviar::ProcessoJaAbertoNoDestino { aberto_em });
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
    let falha = submeter_envio(&mut rede, sessao, &instalacao, &tela, pedido, &feito)?;
    let verificacao = verificar(sessao, &leitura(false), indice, &mut rede);
    let mut feito = conciliar_efeitos(
        &mut rede,
        &cookie,
        link_anotacao.as_deref(),
        feito,
        falha,
        verificacao,
    )?;
    feito.requisicoes = rede.feitas();
    Ok(feito)
}

/// Monta e tenta a tramitação depois da limpeza independente. Se a
/// submissão nem puder sair, o efeito já relido da anotação não se perde.
fn submeter_envio<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    sessao: &SessaoDoProcesso<'_>,
    instalacao: &Instalacao,
    tela: &TelaLida,
    pedido: &PedidoDoEnvio<'_>,
    feito: &ProcessoEnviado,
) -> Result<Option<FalhaDoEnvio>, ErroEnviar> {
    let campos = campos_do_envio(
        &tela.conferida,
        &feito.destino,
        pedido.manter_aberto,
        feito.metodo_remocao == Some(MetodoRemocao::CheckboxNoEnvio),
    );
    let pares: Vec<(&str, &str)> = campos.iter().map(|(n, v)| (*n, v.as_str())).collect();
    let servida = TelaServida {
        url: &tela.url,
        html: &tela.html,
        codificacao: tela.codificacao,
    };
    let submissao = derivar_submissao(&ENVIAR_PROCESSO, instalacao, &servida, "", &pares)
        .map_err(|causa| erro_apos_limpeza(feito, causa))?;
    enviar(rede, sessao, submissao, &feito.id_procedimento).map_err(|erro| match erro {
        ErroEnviar::Formulario(causa) => erro_apos_limpeza(feito, causa),
        outro => outro,
    })
}

/// Só o formulário de envio falhou: um POST de limpeza anterior já foi
/// comprovado por GET. Preserva a causa tipada e as unidades observadas.
fn erro_apos_limpeza(feito: &ProcessoEnviado, causa: ErroSubmissaoFormulario) -> ErroEnviar {
    if feito.metodo_remocao == Some(MetodoRemocao::LimparAnotacao)
        && feito.anotacao_antes == Some(true)
        && feito.anotacao_depois == Some(false)
    {
        ErroEnviar::FormularioAposLimpezaComprovada {
            aberto_em: feito.aberto_em.clone(),
            causa,
        }
    } else {
        ErroEnviar::Formulario(causa)
    }
}

/// Concilia duas provas independentes. Mesmo quando a árvore falha, o GET
/// da anotação pode comprovar sua remoção sem repetir a escrita.
fn conciliar_efeitos<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    cookie: &str,
    link_anotacao: Option<&str>,
    mut feito: ProcessoEnviado,
    falha: Option<FalhaDoEnvio>,
    verificacao: VerificacaoPorLeitura,
) -> Result<ProcessoEnviado, ErroEnviar> {
    if feito.metodo_remocao == Some(MetodoRemocao::CheckboxNoEnvio) {
        let link = link_anotacao.ok_or(ErroEnviar::RemocaoDeAnotacoesSemVerificacao)?;
        feito.anotacao_depois =
            observar_anotacao_depois(rede, cookie, link, &feito.id_procedimento);
    }
    let feito = conciliar(feito, falha, verificacao)?;
    if feito.metodo_remocao == Some(MetodoRemocao::CheckboxNoEnvio) {
        match feito.anotacao_depois {
            Some(false) => {}
            Some(true) => {
                return Err(ErroEnviar::AnotacoesNaoRemovidasAposEnvio {
                    depois: feito.depois,
                });
            }
            None => {
                return Err(ErroEnviar::NaoConfirmadoNaRemocaoDeAnotacoes {
                    depois: feito.depois,
                });
            }
        }
    }
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
) -> Result<crate::arvore_processo::EnderecoDeEscrita, ErroEnviar> {
    let aberta = aberto_em.iter().any(|sigla| sigla == unidade);
    match (enviar, aberta) {
        (Ok(None) | Err(_), false) => Err(ErroEnviar::ProcessoNaoAbertoNaUnidadeDaSessao {
            aberto_em: aberto_em.to_vec(),
        }),
        (Ok(Some(_)), false) => Err(ErroEnviar::UnidadeDaSessaoAusenteDaSituacao {
            aberto_em: aberto_em.to_vec(),
        }),
        (Ok(None), true) => Err(ErroEnviar::EnvioNaoPublicadoNaBarra {
            aberto_em: aberto_em.to_vec(),
        }),
        (Err(erro), true) => Err(ErroEnviar::Barra(erro)),
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
) -> Result<Option<MetodoRemocao>, ErroEnviar> {
    if !pedido.remover_anotacoes {
        return Ok(None);
    }
    let metodo = if tela.remover_anotacoes_oferecida {
        MetodoRemocao::CheckboxNoEnvio
    } else if anotacao_antes.is_some() {
        if !anotacao_lida.is_some_and(|lida| super::anotacoes::salvar_oferecido(&lida.html)) {
            return Err(if pedido.confirmar {
                ErroEnviar::RemocaoDeAnotacoesSemVerificacao
            } else {
                ErroEnviar::RemocaoDeAnotacoesNaoOferecida
            });
        }
        MetodoRemocao::LimparAnotacao
    } else {
        return Err(ErroEnviar::RemocaoDeAnotacoesNaoOferecida);
    };
    if pedido.confirmar && link.is_none() {
        return Err(ErroEnviar::RemocaoDeAnotacoesSemVerificacao);
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
) -> Result<Option<AnotacaoLida>, ErroEnviar> {
    let Some(link) = link else {
        return Ok(None);
    };
    let (html, mime) = obter_tela(rede, cookie, link).map_err(|falha| match falha {
        FalhaDaTela::Transporte(falha) => {
            ErroEnviar::from(super::anotacoes::ErroDasAnotacoes::TelaDasAnotacoes(falha))
        }
        FalhaDaTela::Redirecionada => {
            ErroEnviar::from(super::anotacoes::ErroDasAnotacoes::TelaDasAnotacoesRedirecionada)
        }
        FalhaDaTela::SessaoEncerrada(crate::erro_publico::SessaoEncerrada) => {
            ErroEnviar::SessaoEncerrada(crate::erro_publico::SessaoEncerrada)
        }
    })?;
    let presente = super::anotacoes::anotacao_presente(&html, id)
        .map_err(super::anotacoes::ErroDasAnotacoes::from)?;
    let codificacao = codificacao_da_tela(&mime).ok_or(ErroEnviar::CharsetDaTelaNaoSuportado)?;
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
) -> Result<(), ErroEnviar> {
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
        return Err(ErroEnviar::Formulario(
            resposta.expect_err("sem POST só falha na derivação"),
        ));
    }
    // Mesmo erro de transporte pós-POST não autoriza retry. A leitura decide.
    match conferir_anotacao_depois(rede, cookie, link, id, None) {
        Ok(()) => Ok(()),
        Err(ErroEnviar::AnotacoesNaoRemovidasAposEnvio { .. }) => {
            Err(ErroEnviar::AnotacaoNaoLimpaAntesDeEnviar)
        }
        Err(releitura) => Err(ErroEnviar::NaoConfirmadoNaLimpezaDaAnotacao(Box::new(
            releitura,
        ))),
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
) -> Result<(), ErroEnviar> {
    match observar_anotacao_depois(rede, cookie, link, id) {
        Some(false) => Ok(()),
        Some(true) => Err(ErroEnviar::AnotacoesNaoRemovidasAposEnvio {
            depois: unidades_apos_envio.map(ToOwned::to_owned),
        }),
        None => Err(ErroEnviar::NaoConfirmadoNaRemocaoDeAnotacoes {
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
) -> Result<TelaLida, ErroEnviar> {
    let (html, mime) = obter_tela(rede, cookie, url).map_err(|falha| match falha {
        FalhaDaTela::Transporte(falha) => ErroEnviar::TelaDaEscrita(falha),
        FalhaDaTela::Redirecionada => ErroEnviar::TelaDaEscritaRedirecionada,
        FalhaDaTela::SessaoEncerrada(crate::erro_publico::SessaoEncerrada) => {
            ErroEnviar::SessaoEncerrada(crate::erro_publico::SessaoEncerrada)
        }
    })?;
    let conferida = conferir_tela_de_envio(&html, id)?;
    let codificacao = codificacao_da_tela(&mime).ok_or(ErroEnviar::CharsetDaTelaNaoSuportado)?;
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
) -> Result<Destino, ErroEnviar> {
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
        .map_err(|erro| ErroEnviar::Consulta(FalhaDaConsulta::Derivacao(erro)))?;
        let resposta = rede
            .post(consulta, cookie)
            .map_err(|falha| ErroEnviar::Consulta(FalhaDaConsulta::Transporte(falha)))?;
        ler_resposta_da_consulta(resposta).map_err(ErroEnviar::Consulta)
    })
}

/// Um único POST. `Ok(None)`: o 302 esperado; `Ok(Some(falha))`: o pedido
/// saiu e a resposta não é a esperada; `Err`: nada saiu.
fn enviar<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    sessao: &SessaoDoProcesso<'_>,
    submissao: SubmissaoDerivada,
    id: &str,
) -> Result<Option<FalhaDoEnvio>, ErroEnviar> {
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
        Err(erro) => Err(ErroEnviar::Formulario(erro)),
    }
}

/// Tabela de reconciliação do contrato.
fn conciliar(
    mut feito: ProcessoEnviado,
    falha: Option<FalhaDoEnvio>,
    verificacao: VerificacaoPorLeitura,
) -> Result<ProcessoEnviado, ErroEnviar> {
    let VerificacaoPorLeitura::Lida(depois) = verificacao else {
        return Err(ErroEnviar::NaoConfirmadoNoEnvio {
            aberto_em: feito.aberto_em,
            anotacao_depois: feito.anotacao_depois,
            falha_do_envio: falha,
            verificacao: Box::new(verificacao),
        });
    };
    if !depois.contains(&feito.destino.sigla) {
        return Err(ErroEnviar::EnvioNaoAplicado {
            aberto_em: feito.aberto_em,
            depois,
            anotacao_depois: feito.anotacao_depois,
            falha_do_envio: falha,
        });
    }
    if depois.contains(&feito.unidade) != feito.manter_aberto {
        return Err(ErroEnviar::SituacaoDaOrigemDiferenteDaPedidaAposEnvio {
            aberto_em: feito.aberto_em,
            depois,
            anotacao_depois: feito.anotacao_depois,
            falha_do_envio: falha,
        });
    }
    feito.alterado = true;
    feito.depois = Some(depois);
    feito.reconciliado_por = falha;
    Ok(feito)
}
