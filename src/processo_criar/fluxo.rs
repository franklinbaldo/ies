//! Fluxo Web de criação de processo: prévia sem POST criador; escrita única
//! com 302 e verificação da identidade na árvore. Nunca há retry.
use crate::acao_do_sei::AcaoDoSei;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDeSessao, EstadoDoProcesso,
    OperacaoDeFormulario, RespostaDoEnvio, ResultadoDoEnvio, TelaServida, derivar_submissao,
    enviar_submissao_via,
};
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo_criar_cadastro::{
    ErroCadastro, conferir_cadastro, conferir_cadastro_com_ajax_pendente,
};
use crate::processo_criar_hipotese::{ErroHipotese, resolver as resolver_hipotese};
use crate::processo_criar_interessados::{resolver as resolver_interessados, valor_da_lupa};
use crate::processo_criar_lista::{ErroListaDeTipos, EscolhaDoTipo, tipo_na_lista};
use crate::processo_criar_menu::link_de_iniciar;

use crate::transporte::decodificar_resposta;
use crate::transporte::sessao::CookieDaSessao;
use crate::visao_do_controle::EnvioDoControle;

const FILTRO_TIPOS: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::ProcedimentoEscolherTipo,
    formulario: "frmProcedimentoEscolherTipo",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &["hdnFiltroTipoProcedimento"],
    campos_ocultos_reescritos: &["hdnFiltroTipoProcedimento"],
    campos_usuario: &[],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
        o_que: EstadoDeSessao::PaginaOuFiltroLembrado,
    },
};

const ESCOLHER_TIPO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::ProcedimentoEscolherTipo,
    formulario: "frmProcedimentoEscolherTipo",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &["hdnIdTipoProcedimento"],
    campos_ocultos_reescritos: &["hdnIdTipoProcedimento"],
    campos_usuario: &[],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::ConsultaSemEfeitoObservado {
        evidencia: "docs/como-o-sei-funciona/inicio-do-processo.md: escolher(id) abre o cadastro",
    },
};

const CADASTRAR: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::ProcedimentoGerar,
    formulario: "frmProcedimentoCadastro",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &[
        "hdnIdTipoProcedimento",
        "hdnFlagProcedimentoCadastro",
        "hdnAssuntos",
    ],
    campos_ocultos_reescritos: &[
        "hdnFlagProcedimentoCadastro",
        "hdnAssuntos",
        "hdnInteressadosProcedimento",
    ],
    campos_usuario: &["txtDescricao", "rdoNivelAcesso", "selHipoteseLegal"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::ProcessoCriadoNaUnidade,
    },
};

#[path = "erro_fluxo.rs"]
mod erro_fluxo;
#[path = "links.rs"]
mod links;
#[path = "pos_escrita.rs"]
mod pos_escrita;
#[path = "verificacao_apos_post.rs"]
mod verificacao_apos_post;
pub(crate) use erro_fluxo::ErroCriar;
use links::{id_do_link, resolver_link, retorno_da_criacao, url_do_redirecionamento};

pub(crate) struct PedidoCriar<'a> {
    pub(crate) tipo: &'a str,
    pub(crate) especificacao: &'a str,
    pub(crate) interessados: &'a [String],
    pub(crate) nivel: &'a str,
    pub(crate) unidade: Option<&'a str>,
    pub(crate) hipotese_legal: Option<&'a str>,
    pub(crate) confirmar: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Criacao {
    pub(crate) tipo: String,
    pub(crate) assunto: String,
    pub(crate) nivel: String,
    pub(crate) hipotese_legal: Option<String>,
    pub(crate) unidade: String,
    pub(crate) confirmado: bool,
    pub(crate) interessados: Vec<String>,
    pub(crate) nup: Option<String>,
    pub(crate) causa_post: Option<&'static str>,
    pub(crate) requisicoes: u32,
}

struct Tela {
    url: String,
    html: String,
    mime: String,
}

fn get<V: EnvioDoControle>(
    envio: &mut V,
    url: &str,
    cookie: &str,
    requisicoes: &mut u32,
) -> Result<Tela, ErroCriar> {
    *requisicoes += 1;
    let resposta = envio.get(url, cookie).map_err(ErroCriar::Transporte)?;
    let mime = resposta.mime.clone();
    let html = decodificar_resposta(resposta).map_err(ErroCriar::Transporte)?;
    Ok(Tela {
        url: url.to_owned(),
        html,
        mime,
    })
}

#[allow(clippy::too_many_arguments)] // contexto da sessão e a operação são explícitos.
fn enviada<V: EnvioDoControle>(
    envio: &mut V,
    cookie: &CookieDaSessao,
    operacao: &OperacaoDeFormulario,
    instalacao: &Instalacao,
    tela: &Tela,
    unidade: &str,
    campos: &[(&str, &str)],
    requisicoes: &mut u32,
    eh_escrita: bool,
) -> Result<RespostaDoEnvio, ErroCriar> {
    let codificacao = codificacao_da_tela(&tela.mime).ok_or(ErroCriar::Formulario)?;
    let servida = TelaServida {
        url: &tela.url,
        html: &tela.html,
        codificacao,
    };
    let submissao = derivar_submissao(operacao, instalacao, &servida, unidade, campos)
        .map_err(|_| ErroCriar::Formulario)?;
    let mut tentou = false;
    let resultado = enviar_submissao_via(submissao, cookie, |s, c| {
        tentou = true;
        *requisicoes += 1;
        envio.post(s, c)
    });
    // Após iniciar a escrita, até uma falha de transporte tem efeito incerto.
    resultado.map_err(|erro| {
        if eh_escrita && tentou {
            ErroCriar::NaoConfirmado
        } else {
            ErroCriar::Submissao(erro)
        }
    })
}

/// Compatibilidade dos cenários offline que só verificam o resultado.
#[cfg(test)]
pub(crate) fn criar<V: EnvioDoControle>(
    origem: &str,
    url_controle: &str,
    cookie: &CookieDaSessao,
    pedido: &PedidoCriar<'_>,
    envio: &mut V,
) -> Result<Criacao, ErroCriar> {
    let mut requisicoes = 0;
    criar_com_contagem(
        origem,
        url_controle,
        cookie,
        pedido,
        envio,
        &mut requisicoes,
    )
}

/// Um POST de escolha é leitura do cadastro. O POST com CADASTRAR só
/// acontece após conferência da lista, do formulário e de todos os campos.
#[allow(clippy::too_many_lines)] // fluxo sequencial até a verificação, sem retries.
pub(crate) fn criar_com_contagem<V: EnvioDoControle>(
    origem: &str,
    url_controle: &str,
    cookie: &CookieDaSessao,
    pedido: &PedidoCriar<'_>,
    envio: &mut V,
    requisicoes: &mut u32,
) -> Result<Criacao, ErroCriar> {
    *requisicoes = 0;
    let instalacao = Instalacao::interpretar(origem).ok_or(ErroCriar::SessaoLocalIlegivel)?;
    let cookie_valor = cookie.valor().map_err(|_| ErroCriar::SessaoLocalIlegivel)?;
    let controle = get(envio, url_controle, cookie_valor, requisicoes)?;
    let unidade = crate::controle_reconhecido::unidade_do_controle(&controle.html)
        .map_err(|_| ErroCriar::UnidadeDiferente)?;
    if pedido.unidade.is_some_and(|exigida| exigida != unidade) {
        return Err(ErroCriar::UnidadeDiferente);
    }
    let link =
        link_de_iniciar(&instalacao, &controle.url, &controle.html).map_err(ErroCriar::Menu)?;
    let mut lista = get(envio, &link, cookie_valor, requisicoes)?;
    let escolhido = match tipo_na_lista(&lista.html, pedido.tipo) {
        Err(ErroListaDeTipos::ListaIncompleta) => {
            let resposta = enviada(
                envio,
                cookie,
                &FILTRO_TIPOS,
                &instalacao,
                &lista,
                &unidade,
                &[("hdnFiltroTipoProcedimento", "T")],
                requisicoes,
                false,
            )?;
            let RespostaDoEnvio::Html(html) = resposta else {
                return Err(ErroCriar::Lista(ErroListaDeTipos::ListaIncompleta));
            };
            lista.html = html;
            tipo_na_lista(&lista.html, pedido.tipo).map_err(ErroCriar::Lista)?
        }
        outro => outro.map_err(ErroCriar::Lista)?,
    };
    let (id_tipo, cadastro) = match escolhido.escolha {
        EscolhaDoTipo::Link(link) => {
            let id_tipo = id_do_link(&link)
                .ok_or(ErroCriar::TipoSemIdentificador)?
                .to_owned();
            let url = resolver_link(&instalacao, &lista.url, &link, AcaoDoSei::ProcedimentoGerar)?;
            (id_tipo, get(envio, &url, cookie_valor, requisicoes)?)
        }
        EscolhaDoTipo::Formulario(id_tipo) => {
            let resposta = enviada(
                envio,
                cookie,
                &ESCOLHER_TIPO,
                &instalacao,
                &lista,
                &unidade,
                &[("hdnIdTipoProcedimento", &id_tipo)],
                requisicoes,
                false,
            )?;
            let RespostaDoEnvio::Redirecionamento(redirecionamento) = resposta else {
                return Err(ErroCriar::EscolhaInesperada);
            };
            let url = url_do_redirecionamento(
                &instalacao,
                redirecionamento
                    .localizacao()
                    .ok_or(ErroCriar::EscolhaInesperada)?,
                redirecionamento.url_do_envio(),
                AcaoDoSei::ProcedimentoGerar,
            )
            .map_err(|_| ErroCriar::EscolhaInesperada)?;
            (id_tipo, get(envio, &url, cookie_valor, requisicoes)?)
        }
    };
    let mut observado = match conferir_cadastro(&cadastro.html, &id_tipo, pedido.nivel) {
        Ok(observado) => observado,
        Err(ErroCadastro::HipoteseLegalNaoIdentificada) if pedido.nivel == "restrito" => {
            conferir_cadastro_com_ajax_pendente(&cadastro.html, &id_tipo, pedido.nivel)
                .map_err(ErroCriar::Cadastro)?
        }
        Err(erro) => return Err(ErroCriar::Cadastro(erro)),
    };
    // #1248 (Todos): a prévia e a confirmação devem respeitar o limite
    // de txtDescricao que o cadastro desta instalação publicou.
    // Recusar antes do POST criador (e antes de consultas AJAX adicionais).
    observado
        .validar_especificacao(pedido.especificacao)
        .map_err(ErroCriar::Cadastro)?;
    let codificacao = codificacao_da_tela(&cadastro.mime).ok_or(ErroCriar::Formulario)?;
    let servida = TelaServida {
        url: &cadastro.url,
        html: &cadastro.html,
        codificacao,
    };
    // A escolha explícita é autorizada só para a lista AJAX observada.
    // Uma opção inline não tem seu texto provado por essa mesma leitura.
    if pedido.hipotese_legal.is_some() && observado.hipotese_legal.is_some() {
        return Err(ErroCriar::Hipotese(
            ErroHipotese::EscolhaInlineNaoVerificada,
        ));
    }
    if pedido.nivel == "restrito" && observado.hipotese_legal.is_none() {
        observado.hipotese_legal = Some(
            resolver_hipotese(
                envio,
                &instalacao,
                &servida,
                cookie_valor,
                requisicoes,
                pedido.hipotese_legal,
            )
            .map_err(ErroCriar::Hipotese)?,
        );
    }
    let interessados = resolver_interessados(
        envio,
        &instalacao,
        &servida,
        cookie_valor,
        pedido.interessados,
        requisicoes,
    )
    .map_err(ErroCriar::Interessado)?;
    let mut feito = Criacao {
        tipo: escolhido.nome,
        assunto: observado.assunto.clone(),
        nivel: pedido.nivel.to_owned(),
        hipotese_legal: observado.hipotese_legal.clone(),
        unidade,
        confirmado: false,
        interessados: interessados.iter().map(|(_, nome)| nome.clone()).collect(),
        nup: None,
        causa_post: None,
        requisicoes: *requisicoes,
    };
    let lupa = format!("{}±{}", observado.id_assunto, observado.assunto);
    let nivel = if pedido.nivel == "publico" { "0" } else { "1" };
    let mut campos = vec![
        ("hdnFlagProcedimentoCadastro", "2"),
        ("hdnAssuntos", lupa.as_str()),
        ("txtDescricao", pedido.especificacao),
        ("rdoNivelAcesso", nivel),
    ];
    if let Some(hipotese) = observado.hipotese_legal.as_deref() {
        campos.push(("selHipoteseLegal", hipotese));
    }
    let codificados = valor_da_lupa(&interessados);
    if !interessados.is_empty() {
        campos.push(("hdnInteressadosProcedimento", codificados.as_str()));
    }
    // Derive o POST inclusive na prévia, sem executar JS e sem escrita.
    let submissao = derivar_submissao(&CADASTRAR, &instalacao, &servida, &feito.unidade, &campos)
        .map_err(|_| ErroCriar::Formulario)?;
    if !pedido.confirmar {
        return Ok(feito);
    }
    // A fotografia dos Gerados *antes* do POST só serve para reconciliar
    // uma resposta incerta. Sem cobertura integral (conta compartilhada com
    // mais de uma página, PEN1 10/11/2026), a escrita segue: o Location
    // continua a ser a prova, e uma resposta incerta sai com efeito incerto.
    // Esta leitura usa o Controle que já foi obtido: zero novos requests.
    let gerados_antes =
        match reconciliacao::FotografiaDosGerados::da_tela(&controle.html, &feito.unidade) {
            Ok(fotografia) => Some(fotografia),
            Err(reconciliacao::ErroFotografia::CoberturaIncompleta) => None,
            Err(erro) => return Err(ErroCriar::Fotografia(erro)),
        };
    // Não há retry após a submissão.
    let mut tentou = false;
    let resposta = match enviar_submissao_via(submissao, cookie, |s, c| {
        tentou = true;
        *requisicoes += 1;
        envio.post(s, c)
    }) {
        Ok(resposta) => resposta,
        // Recusa explícita não pode virar sucesso por candidato idêntico de
        // outro agente na conta compartilhada. Não consultar Gerados depois.
        Err(causa_post)
            if tentou
                && matches!(
                    &causa_post,
                    crate::formulario_web::ErroSubmissaoFormulario::ValidacaoRecusadaPeloSei(_)
                ) =>
        {
            return Err(ErroCriar::RecusaDoPost(causa_post));
        }
        // HTTP 4xx isolado pode vir de intermediário: nem prova de recusa
        // do SEI nem permissão para atribuir um candidato concorrente.
        Err(causa_post)
            if tentou
                && causa_post
                    .status_http()
                    .is_some_and(|status| (400..500).contains(&status.status)) =>
        {
            return Err(ErroCriar::RespostaHttpSemProva(causa_post));
        }
        Err(causa_post) if tentou => {
            pos_escrita::reconciliar(
                envio,
                &instalacao,
                url_controle,
                cookie_valor,
                &mut feito,
                gerados_antes.as_ref(),
                pedido.especificacao,
                requisicoes,
                pos_escrita::CausaPosEscrita::Transporte(causa_post),
            )?;
            return Ok(feito);
        }
        Err(_) => return Err(ErroCriar::Formulario),
    };
    let RespostaDoEnvio::Redirecionamento(redirecionamento) = resposta else {
        pos_escrita::reconciliar(
            envio,
            &instalacao,
            url_controle,
            cookie_valor,
            &mut feito,
            gerados_antes.as_ref(),
            pedido.especificacao,
            requisicoes,
            pos_escrita::CausaPosEscrita::Resposta(
                erro_fluxo::CausaRespostaCriadora::RespostaInesperada,
            ),
        )?;
        return Ok(feito);
    };
    let retorno = redirecionamento
        .localizacao()
        .ok_or(ErroCriar::NaoConfirmado)
        .and_then(|localizacao| {
            retorno_da_criacao(&instalacao, localizacao, redirecionamento.url_do_envio())
        });
    let Ok((url, id)) = retorno else {
        pos_escrita::reconciliar(
            envio,
            &instalacao,
            url_controle,
            cookie_valor,
            &mut feito,
            gerados_antes.as_ref(),
            pedido.especificacao,
            requisicoes,
            pos_escrita::CausaPosEscrita::Resposta(
                erro_fluxo::CausaRespostaCriadora::LocalizacaoInvalida,
            ),
        )?;
        return Ok(feito);
    };
    // O Location legítimo também não comprova uma criação nova quando
    // aponta a ID que já existia nos Gerados antes do único POST.
    // Recusar sem abrir processo preexistente nem repetir a escrita.
    if gerados_antes
        .as_ref()
        .is_some_and(|antes| antes.contem_id(&id))
    {
        return Err(ErroCriar::VerificacaoAposPost(Box::new(
            ErroCriar::ProvaDaCriacao(erro_fluxo::ProvaDaCriacao::IdentidadePreexistente),
        )));
    }
    // O Location legítimo não prova o efeito da criação. Preservar a
    // causa da verificação, inclusive status HTTP, sem fazer novo POST.
    let nup = verificacao_apos_post::verificar(
        envio,
        &instalacao,
        &url,
        &id,
        cookie_valor,
        &feito,
        pedido,
        requisicoes,
    )
    .map_err(|erro| ErroCriar::VerificacaoAposPost(Box::new(erro)))?;
    feito.confirmado = true;
    feito.nup = Some(nup);
    feito.requisicoes = *requisicoes;
    Ok(feito)
}

#[cfg(test)]
#[path = "tests_fluxo.rs"]
mod tests;

#[cfg(test)]
#[path = "tests_resposta_post.rs"]
mod tests_resposta_post;

#[cfg(test)]
#[path = "tests_hipotese.rs"]
mod tests_hipotese;

#[cfg(test)]
#[path = "acervo.rs"]
mod acervo;

#[path = "reconciliacao.rs"]
mod reconciliacao;

#[cfg(test)]
#[path = "tests_candidatos.rs"]
mod tests_candidatos;

#[cfg(test)]
#[path = "tests_concorrencia.rs"]
mod tests_concorrencia;

#[cfg(test)]
#[path = "tests_contagem.rs"]
mod tests_contagem;

#[cfg(test)]
#[path = "tests_select_desabilitado.rs"]
mod tests_select_desabilitado;
