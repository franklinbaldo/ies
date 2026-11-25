//! A sequência de `ies processo enviar` (contracts/cli/processo-enviar.md):
//! abertura, decisão antes da tela, a tela, a consulta do autocomplete, o
//! destino, um POST e a leitura de verificação, com a tabela de
//! reconciliação do contrato. Nenhum 302 é seguido; nenhuma escrita é
//! repetida.
use super::destino::{ler_resposta_da_consulta, resolver_destino};
use super::erro::{ErroEnviar, FalhaDaConsulta};
use super::releitura_anotacao::{
    conferir_anotacao_depois, observar_anotacao_depois, obter_tela_das_anotacoes,
};
use super::tela::{
    AUTOCOMPLETAR_UNIDADE, ENVIAR_PROCESSO, TelaDeEnvio, campos_da_consulta_no_orgao,
    campos_do_envio, conferir_tela_de_envio, redirecionamento_esperado,
};
use super::{
    DesfechoDoEnvio, Destino, MetodoRemocao, PedidoDoEnvio, ProcessoEnviado, RemocaoDaAnotacao,
};
use crate::formulario_web::{
    CodificacaoDaTela, ErroSubmissaoFormulario, RespostaDoEnvio, SubmissaoDerivada, TelaServida,
    derivar_consulta_ajax, derivar_submissao, enviar_submissao_via,
};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo_concluir::escrita::{
    Contador, FalhaDaTela, leitura, obter_tela, siglas_abertas, verificar,
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
    html: crate::tela_html::HtmlServido,
    codificacao: CodificacaoDaTela,
    conferida: TelaDeEnvio,
}

/// Mesmo formulário usado para observar e, quando autorizado, limpar.
struct AnotacaoLida {
    presente: bool,
    html: crate::tela_html::HtmlServido,
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
    let id = aberto.arvore.processo.id;
    let cookie = acoes.cookie.clone();
    let tela = ler_tela(&mut rede, &cookie, link.as_str(), &id)?;
    let (link_anotacao, anotacao_lida, metodo_remocao) = if pedido.remover_anotacoes {
        // Sem a tela de anotações a presença antes não é lida: a remoção
        // não é oferecida na prévia nem conferível na confirmação.
        let Some(link) = acoes
            .anotacoes
            .map_err(super::anotacoes::ErroDasAnotacoes::LinkDasAnotacoes)?
        else {
            return Err(if pedido.confirmar {
                ErroEnviar::RemocaoDeAnotacoesSemVerificacao
            } else {
                ErroEnviar::RemocaoDeAnotacoesNaoOferecida
            });
        };
        let lida = ler_anotacao_antes(&mut rede, &cookie, &link, &id)?;
        let metodo = escolher_metodo(pedido, &tela.conferida, &lida)?;
        (Some(link), Some(lida), Some(metodo))
    } else {
        (None, None, None)
    };
    let destino = consultar_destino(&mut rede, &cookie, &instalacao, &tela, pedido.para)?;
    if aberto_em.contains(&destino.sigla) {
        return Err(ErroEnviar::ProcessoJaAbertoNoDestino { aberto_em });
    }
    let feito = ProcessoEnviado {
        nup: pedido.nup.as_str().to_owned(),
        id_procedimento: id,
        unidade,
        manter_aberto: pedido.manter_aberto,
        remocao: metodo_remocao
            .zip(anotacao_lida.as_ref())
            .map(|(metodo, lida)| RemocaoDaAnotacao {
                metodo,
                antes: lida.presente,
            }),
        anotacao_depois: None,
        destino,
        aberto_em,
        desfecho: DesfechoDoEnvio::Previa,
        requisicoes: rede.feitas(),
    };
    if !pedido.confirmar {
        return Ok(feito);
    }
    let feito = limpar_se_publicada(
        &mut rede,
        sessao,
        &instalacao,
        &cookie,
        link_anotacao.as_deref(),
        anotacao_lida.as_ref(),
        feito,
    )?;
    let falha = submeter_envio(&mut rede, sessao, &instalacao, &tela, pedido, &feito)?;
    let verificacao = verificar(sessao, &leitura(false), indice, &mut rede);
    let feito = conciliar_efeitos(
        &mut rede,
        &cookie,
        link_anotacao.as_deref(),
        feito,
        falha,
        verificacao,
    )?;
    Ok(ProcessoEnviado {
        requisicoes: rede.feitas(),
        ..feito
    })
}

/// BDD-333-03/04: grava anotação vazia somente se havia texto. Cada POST é
/// único e a releitura precisa comprovar a limpeza antes do envio.
fn limpar_se_publicada<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    sessao: &SessaoDoProcesso<'_>,
    instalacao: &Instalacao,
    cookie: &str,
    link: Option<&str>,
    lida: Option<&AnotacaoLida>,
    feito: ProcessoEnviado,
) -> Result<ProcessoEnviado, ErroEnviar> {
    if feito.metodo_remocao() != Some(MetodoRemocao::LimparAnotacao) {
        return Ok(feito);
    }
    let (Some(link), Some(lida)) = (link, lida) else {
        return Err(ErroEnviar::RemocaoDeAnotacoesSemVerificacao);
    };
    if lida.presente {
        limpar_anotacao(
            rede,
            sessao,
            instalacao,
            cookie,
            link,
            lida,
            &feito.id_procedimento,
        )?;
    }
    Ok(feito.com_anotacao_depois(false))
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
        feito.metodo_remocao() == Some(MetodoRemocao::CheckboxNoEnvio),
    );
    let pares: Vec<(&str, &str)> = campos.iter().map(|(n, v)| (*n, v.as_str())).collect();
    let servida = TelaServida {
        url: &tela.url,
        html: &tela.html,
        codificacao: tela.codificacao,
    };
    let submissao = derivar_submissao(&ENVIAR_PROCESSO, instalacao, &servida, None, &pares)
        .map_err(|causa| erro_apos_limpeza(feito, causa))?;
    enviar(rede, sessao, submissao, &feito.id_procedimento).map_err(|erro| match erro {
        ErroEnviar::Formulario(causa) => erro_apos_limpeza(feito, causa),
        outro @ (ErroEnviar::Abertura(_)
        | ErroEnviar::UnidadeDaSessaoNaoPublicada
        | ErroEnviar::SituacaoDoProcessoNaoLidaAntesDeEnviar { .. }
        | ErroEnviar::ProcessoNaoAbertoNaUnidadeDaSessao { .. }
        | ErroEnviar::UnidadeDaSessaoAusenteDaSituacao { .. }
        | ErroEnviar::EnvioNaoPublicadoNaBarra { .. }
        | ErroEnviar::Barra(_)
        | ErroEnviar::TelaDaEscrita(_)
        | ErroEnviar::TelaDaEscritaRedirecionada
        | ErroEnviar::SessaoEncerrada(_)
        | ErroEnviar::CharsetDaTelaNaoSuportado
        | ErroEnviar::TelaDeEnvio(_)
        | ErroEnviar::Consulta(_)
        | ErroEnviar::RemocaoDeAnotacoesNaoOferecida
        | ErroEnviar::RemocaoDeAnotacoesSemVerificacao
        | ErroEnviar::AnotacoesNaoRemovidasAposEnvio { .. }
        | ErroEnviar::AnotacaoNaoLimpaAntesDeEnviar
        | ErroEnviar::NaoConfirmadoNaLimpezaDaAnotacao(_)
        | ErroEnviar::NaoConfirmadoNaRemocaoDeAnotacoes { .. }
        | ErroEnviar::Anotacoes(_)
        | ErroEnviar::DestinoNaoOferecidoPeloSei { .. }
        | ErroEnviar::VariosDestinosComASigla { .. }
        | ErroEnviar::ProcessoJaAbertoNoDestino { .. }
        | ErroEnviar::FormularioAposLimpezaComprovada { .. }
        | ErroEnviar::EnvioNaoAplicado { .. }
        | ErroEnviar::SituacaoDaOrigemDiferenteDaPedidaAposEnvio { .. }
        | ErroEnviar::NaoConfirmadoNoEnvio { .. }) => outro,
    })
}

/// Só o formulário de envio falhou: um POST de limpeza anterior já foi
/// comprovado por GET. Preserva a causa tipada e as unidades observadas.
fn erro_apos_limpeza(feito: &ProcessoEnviado, causa: ErroSubmissaoFormulario) -> ErroEnviar {
    let limpeza_comprovada = matches!(
        (&feito.remocao, feito.anotacao_depois),
        (
            Some(RemocaoDaAnotacao {
                metodo: MetodoRemocao::LimparAnotacao,
                antes: true,
            }),
            Some(false),
        )
    );
    if limpeza_comprovada {
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
    feito: ProcessoEnviado,
    falha: Option<FalhaDoEnvio>,
    verificacao: VerificacaoPorLeitura,
) -> Result<ProcessoEnviado, ErroEnviar> {
    let observacao = if feito.metodo_remocao() == Some(MetodoRemocao::CheckboxNoEnvio) {
        let link = link_anotacao.ok_or(ErroEnviar::RemocaoDeAnotacoesSemVerificacao)?;
        Some(observar_anotacao_depois(
            rede,
            cookie,
            link,
            &feito.id_procedimento,
        ))
    } else {
        None
    };
    // Preservar o que a anotação mostrou mesmo se a árvore ficar ilegível.
    let feito = if let Some(Ok(presente)) = &observacao {
        feito.com_anotacao_depois(*presente)
    } else {
        feito
    };
    let feito = conciliar(feito, falha, verificacao)?;
    match observacao {
        None | Some(Ok(false)) => Ok(feito),
        Some(Ok(true)) => Err(ErroEnviar::AnotacoesNaoRemovidasAposEnvio {
            depois: feito.depois().map(<[String]>::to_vec),
        }),
        Some(Err(causa)) => Err(ErroEnviar::NaoConfirmadoNaRemocaoDeAnotacoes {
            depois: feito.depois().map(<[String]>::to_vec),
            causa: Box::new(causa),
        }),
    }
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
        (Err(erro), _) => Err(ErroEnviar::Barra(erro)),
        (Ok(None), false) => Err(ErroEnviar::ProcessoNaoAbertoNaUnidadeDaSessao {
            aberto_em: aberto_em.to_vec(),
        }),
        (Ok(Some(_)), false) => Err(ErroEnviar::UnidadeDaSessaoAusenteDaSituacao {
            aberto_em: aberto_em.to_vec(),
        }),
        (Ok(None), true) => Err(ErroEnviar::EnvioNaoPublicadoNaBarra {
            aberto_em: aberto_em.to_vec(),
        }),
        (Ok(Some(link)), true) => Ok(link),
    }
}

/// Decide a estratégia publicada na tela sem confundir leitura com gravação.
/// Só é chamada quando a remoção foi pedida e a tela de anotações foi lida.
fn escolher_metodo(
    pedido: &PedidoDoEnvio<'_>,
    tela: &TelaDeEnvio,
    lida: &AnotacaoLida,
) -> Result<MetodoRemocao, ErroEnviar> {
    if tela.remover_anotacoes_oferecida {
        return Ok(MetodoRemocao::CheckboxNoEnvio);
    }
    if super::anotacoes::salvar_oferecido(&lida.html)
        .map_err(super::anotacoes::ErroDasAnotacoes::from)?
    {
        return Ok(MetodoRemocao::LimparAnotacao);
    }
    Err(if pedido.confirmar {
        ErroEnviar::RemocaoDeAnotacoesSemVerificacao
    } else {
        ErroEnviar::RemocaoDeAnotacoesNaoOferecida
    })
}

/// Lê a presença da anotação da origem por GET independente. O campo vazio
/// só é ausência comprovada quando o formulário pertence ao mesmo processo.
fn ler_anotacao_antes<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    cookie: &str,
    link: &str,
    id: &str,
) -> Result<AnotacaoLida, ErroEnviar> {
    let (html, mime) = obter_tela_das_anotacoes(rede, cookie, link)?;
    let presente = super::anotacoes::anotacao_presente(&html, id)
        .map_err(super::anotacoes::ErroDasAnotacoes::from)?;
    let codificacao = codificacao_da_tela(&mime).ok_or(ErroEnviar::CharsetDaTelaNaoSuportado)?;
    Ok(AnotacaoLida {
        presente,
        html,
        codificacao,
    })
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
        None,
        &[("txaDescricao", "")],
    )?;
    let mut saiu = false;
    let resposta = enviar_submissao_via(submissao, sessao.cookie, |s, c| {
        saiu = true;
        rede.post(s, c)
    });
    if !saiu && let Err(erro) = resposta {
        return Err(ErroEnviar::Formulario(erro));
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
    feito: ProcessoEnviado,
    falha: Option<FalhaDoEnvio>,
    verificacao: VerificacaoPorLeitura,
) -> Result<ProcessoEnviado, ErroEnviar> {
    let anotacao_depois = feito.anotacao_depois();
    let VerificacaoPorLeitura::Lida(depois) = verificacao else {
        return Err(ErroEnviar::NaoConfirmadoNoEnvio {
            aberto_em: feito.aberto_em,
            anotacao_depois,
            falha_do_envio: falha,
            verificacao: Box::new(verificacao),
        });
    };
    if !depois.contains(&feito.destino.sigla) {
        return Err(ErroEnviar::EnvioNaoAplicado {
            aberto_em: feito.aberto_em,
            depois,
            anotacao_depois,
            falha_do_envio: falha,
        });
    }
    if depois.contains(&feito.unidade) != feito.manter_aberto {
        return Err(ErroEnviar::SituacaoDaOrigemDiferenteDaPedidaAposEnvio {
            aberto_em: feito.aberto_em,
            depois,
            anotacao_depois,
            falha_do_envio: falha,
        });
    }
    Ok(feito.enviado(depois, falha))
}
