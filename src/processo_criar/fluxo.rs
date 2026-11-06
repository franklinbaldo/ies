//! Fluxo Web de criação de processo: prévia sem POST criador; escrita única
//! com 302 e verificação da identidade na árvore. Nunca há retry.
use crate::erro_html::ErroHtml;
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
use crate::processo_criar_interessados::{
    ErroInteressado, resolver as resolver_interessados, valor_da_lupa,
};
use crate::processo_criar_lista::{ErroListaDeTipos, EscolhaDoTipo, tipo_na_lista};
use crate::processo_criar_menu::link_de_iniciar;
use crate::transporte::sessao::CookieDaSessao;
use crate::transporte::{FalhaTransporte, StatusHttpRecusado, decodificar_resposta};
use crate::visao_do_controle::EnvioDoControle;

const FILTRO_TIPOS: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "procedimento_escolher_tipo",
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
    acao: "procedimento_escolher_tipo",
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
    acao: "procedimento_gerar",
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ErroCriar {
    /// Falha antes da escrita; guarda o subcódigo do transporte e, quando
    /// houve, o status e a mensagem saneada do SEI.
    Transporte {
        subcodigo: &'static str,
        http: Option<StatusHttpRecusado>,
    },
    UnidadeDiferente,
    Menu(ErroHtml),
    Lista(ErroListaDeTipos),
    Cadastro(ErroCadastro),
    Formulario,
    TipoSemIdentificador,
    EscolhaInesperada,
    Interessado(ErroInteressado),
    Hipotese(ErroHipotese),
    EfeitoIncerto,
}

impl ErroCriar {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::Transporte { subcodigo, .. } => subcodigo,
            Self::UnidadeDiferente => "unidade_da_criacao_divergente",
            Self::Menu(erro) => erro.onde,
            Self::Lista(erro) => erro.subcodigo(),
            Self::Cadastro(erro) => erro.subcodigo(),
            Self::Formulario => "formulario_de_criacao_nao_reconhecido",
            Self::TipoSemIdentificador => "tipo_de_processo_sem_identificador",
            Self::EscolhaInesperada => "cadastro_nao_devolvido_apos_escolha",
            Self::Interessado(erro) => erro.subcodigo(),
            Self::Hipotese(erro) => erro.subcodigo(),
            Self::EfeitoIncerto => "efeito_incerto_na_criacao",
        }
    }

    pub(crate) fn opcoes_para_escolha(&self) -> Option<&[(String, String)]> {
        match self {
            Self::Hipotese(erro) => erro.opcoes_para_escolha(),
            _ => None,
        }
    }

    pub(crate) fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Transporte { http, .. } => http.as_ref(),
            Self::Hipotese(erro) => erro.status_http(),
            _ => None,
        }
    }

    fn de_transporte(falha: &FalhaTransporte) -> Self {
        Self::Transporte {
            subcodigo: falha.subcodigo(),
            http: falha.status_http().cloned(),
        }
    }

    fn local() -> Self {
        Self::Transporte {
            subcodigo: "falha_de_transporte_na_criacao",
            http: None,
        }
    }

    pub(crate) fn saida(&self) -> u8 {
        if *self == Self::EfeitoIncerto { 3 } else { 1 }
    }
}

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
    let resposta = envio
        .get(url, cookie)
        .map_err(|f| ErroCriar::de_transporte(&f))?;
    let mime = resposta.mime.clone();
    let html = decodificar_resposta(resposta).map_err(|f| ErroCriar::de_transporte(&f))?;
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
            ErroCriar::EfeitoIncerto
        } else {
            ErroCriar::Transporte {
                subcodigo: erro.subcodigo(),
                http: erro.status_http().cloned(),
            }
        }
    })
}

fn id_do_link(link: &str) -> Option<&str> {
    let mut ids =
        link.split('?').nth(1)?.split('&').filter(|par| {
            par.split_once('=').map_or(*par, |(nome, _)| nome) == "id_tipo_procedimento"
        });
    let primeiro = ids.next()?.strip_prefix("id_tipo_procedimento=")?;
    (ids.next().is_none() && !primeiro.is_empty() && primeiro.bytes().all(|b| b.is_ascii_digit()))
        .then_some(primeiro)
}

/// Exige exatamente uma ocorrência do parâmetro acao, com valor literal
/// esperado. Uma segunda ação diferente também torna a URL ambígua.
fn acao_unica(consulta: &str, esperada: &str) -> bool {
    let mut acoes = consulta.split('&').filter_map(|par| {
        let (nome, valor) = par.split_once('=').unwrap_or((par, ""));
        (nome == "acao").then_some(valor)
    });
    acoes.next() == Some(esperada) && acoes.next().is_none()
}

fn resolver_link(
    instalacao: &Instalacao,
    base: &str,
    referencia: &str,
    acao: &str,
) -> Result<String, ErroCriar> {
    if !referencia.starts_with("controlador.php?")
        || referencia.contains(['#', '\\', '\''])
        || !referencia.bytes().all(|b| (33..=126).contains(&b))
    {
        return Err(ErroCriar::Formulario);
    }
    let base = base.split('?').next().ok_or(ErroCriar::Formulario)?;
    let (pasta, _) = base.rsplit_once('/').ok_or(ErroCriar::Formulario)?;
    let url = format!("{pasta}/{referencia}");
    let partes = crate::url_observada::UrlHttps::interpretar(&url).ok_or(ErroCriar::Formulario)?;
    if !instalacao.autoriza_controlador(partes.host, partes.caminho) {
        return Err(ErroCriar::Formulario);
    }
    if !acao_unica(partes.consulta.unwrap_or_default(), acao) {
        return Err(ErroCriar::Formulario);
    }
    Ok(url)
}

fn url_do_redirecionamento(
    instalacao: &Instalacao,
    localizacao: &str,
    url_envio: &str,
    acao: &str,
) -> Result<String, ErroCriar> {
    let absoluta = if localizacao.starts_with("https://") {
        localizacao.to_owned()
    } else {
        resolver_link(instalacao, url_envio, localizacao, acao)?
    };
    let partes =
        crate::url_observada::UrlHttps::interpretar(&absoluta).ok_or(ErroCriar::EfeitoIncerto)?;
    if !instalacao.autoriza_controlador(partes.host, partes.caminho) {
        return Err(ErroCriar::EfeitoIncerto);
    }
    if !acao_unica(partes.consulta.unwrap_or_default(), acao) {
        return Err(ErroCriar::EfeitoIncerto);
    }
    Ok(absoluta)
}

fn retorno_da_criacao(
    instalacao: &Instalacao,
    localizacao: &str,
    url_envio: &str,
) -> Result<(String, String), ErroCriar> {
    let url = url_do_redirecionamento(instalacao, localizacao, url_envio, "procedimento_trabalhar")
        .map_err(|_| ErroCriar::EfeitoIncerto)?;
    let consulta = crate::url_observada::UrlHttps::interpretar(&url)
        .and_then(|p| p.consulta)
        .ok_or(ErroCriar::EfeitoIncerto)?;
    let mut ids = consulta
        .split('&')
        .filter(|par| par.split_once('=').map_or(*par, |(nome, _)| nome) == "id_procedimento");
    let primeiro = ids.next().ok_or(ErroCriar::EfeitoIncerto)?;
    if ids.next().is_some() {
        return Err(ErroCriar::EfeitoIncerto);
    }
    let id = primeiro
        .strip_prefix("id_procedimento=")
        .filter(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()))
        .ok_or(ErroCriar::EfeitoIncerto)?
        .to_owned();
    Ok((url, id))
}

/// Um POST de escolha é leitura do cadastro. O POST com CADASTRAR só
/// acontece após conferência da lista, do formulário e de todos os campos.
#[allow(clippy::too_many_lines)] // fluxo sequencial até a verificação, sem retries.
pub(crate) fn criar<V: EnvioDoControle>(
    origem: &str,
    url_controle: &str,
    cookie: &CookieDaSessao,
    pedido: &PedidoCriar<'_>,
    envio: &mut V,
) -> Result<Criacao, ErroCriar> {
    use ErroCriar as E;
    let instalacao = Instalacao::interpretar(origem).ok_or_else(E::local)?;
    let cookie_valor = cookie.valor().map_err(|_| E::local())?;
    let mut requisicoes = 0;
    let controle = get(envio, url_controle, cookie_valor, &mut requisicoes)?;
    let unidade = crate::controle_reconhecido::unidade_do_controle(&controle.html)
        .map_err(|_| E::UnidadeDiferente)?;
    if pedido.unidade.is_some_and(|exigida| exigida != unidade) {
        return Err(E::UnidadeDiferente);
    }
    let link = link_de_iniciar(&instalacao, &controle.url, &controle.html).map_err(E::Menu)?;
    let mut lista = get(envio, &link, cookie_valor, &mut requisicoes)?;
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
                &mut requisicoes,
                false,
            )?;
            let RespostaDoEnvio::Html(html) = resposta else {
                return Err(E::Lista(ErroListaDeTipos::ListaIncompleta));
            };
            lista.html = html;
            tipo_na_lista(&lista.html, pedido.tipo).map_err(E::Lista)?
        }
        outro => outro.map_err(E::Lista)?,
    };
    let (id_tipo, cadastro) = match escolhido.escolha {
        EscolhaDoTipo::Link(link) => {
            let id_tipo = id_do_link(&link).ok_or(E::TipoSemIdentificador)?.to_owned();
            let url = resolver_link(&instalacao, &lista.url, &link, "procedimento_gerar")?;
            (id_tipo, get(envio, &url, cookie_valor, &mut requisicoes)?)
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
                &mut requisicoes,
                false,
            )?;
            let RespostaDoEnvio::Redirecionamento(redirecionamento) = resposta else {
                return Err(E::EscolhaInesperada);
            };
            let url = url_do_redirecionamento(
                &instalacao,
                redirecionamento.localizacao().ok_or(E::EscolhaInesperada)?,
                redirecionamento.url_do_envio(),
                "procedimento_gerar",
            )
            .map_err(|_| E::EscolhaInesperada)?;
            (id_tipo, get(envio, &url, cookie_valor, &mut requisicoes)?)
        }
    };
    let mut observado = match conferir_cadastro(&cadastro.html, &id_tipo, pedido.nivel) {
        Ok(observado) => observado,
        Err(ErroCadastro::HipoteseLegalNaoIdentificada) if pedido.nivel == "restrito" => {
            conferir_cadastro_com_ajax_pendente(&cadastro.html, &id_tipo, pedido.nivel)
                .map_err(E::Cadastro)?
        }
        Err(erro) => return Err(E::Cadastro(erro)),
    };
    let codificacao = codificacao_da_tela(&cadastro.mime).ok_or(E::Formulario)?;
    let servida = TelaServida {
        url: &cadastro.url,
        html: &cadastro.html,
        codificacao,
    };
    // A escolha explícita é autorizada só para a lista AJAX observada.
    // Uma opção inline não tem seu texto provado por essa mesma leitura.
    if pedido.hipotese_legal.is_some() && observado.hipotese_legal.is_some() {
        return Err(E::Hipotese(ErroHipotese::EscolhaInlineNaoVerificada));
    }
    if pedido.nivel == "restrito" && observado.hipotese_legal.is_none() {
        observado.hipotese_legal = Some(
            resolver_hipotese(
                envio,
                &instalacao,
                &servida,
                cookie_valor,
                &mut requisicoes,
                pedido.hipotese_legal,
            )
            .map_err(E::Hipotese)?,
        );
    }
    let interessados = resolver_interessados(
        envio,
        &instalacao,
        &servida,
        cookie_valor,
        pedido.interessados,
        &mut requisicoes,
    )
    .map_err(E::Interessado)?;
    let mut feito = Criacao {
        tipo: escolhido.nome,
        assunto: observado.assunto.clone(),
        nivel: pedido.nivel.to_owned(),
        hipotese_legal: observado.hipotese_legal.clone(),
        unidade,
        confirmado: false,
        interessados: interessados.iter().map(|(_, nome)| nome.clone()).collect(),
        nup: None,
        requisicoes,
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
        .map_err(|_| E::Formulario)?;
    if !pedido.confirmar {
        return Ok(feito);
    }
    // Não há retry após a submissão.
    let mut tentou = false;
    let resposta = enviar_submissao_via(submissao, cookie, |s, c| {
        tentou = true;
        requisicoes += 1;
        envio.post(s, c)
    })
    .map_err(|_| {
        if tentou {
            E::EfeitoIncerto
        } else {
            E::Formulario
        }
    })?;
    let RespostaDoEnvio::Redirecionamento(redirecionamento) = resposta else {
        return Err(E::EfeitoIncerto);
    };
    let (url, id) = retorno_da_criacao(
        &instalacao,
        redirecionamento.localizacao().ok_or(E::EfeitoIncerto)?,
        redirecionamento.url_do_envio(),
    )?;
    // Uma leitura fresca para obter NUP e verificar id; não repetir o POST.
    let tela_do_processo =
        get(envio, &url, cookie_valor, &mut requisicoes).map_err(|_| E::EfeitoIncerto)?;
    let tela_lida = crate::arvore_processo::ler_tela_do_processo(
        &instalacao,
        &tela_do_processo.html,
        &id,
        Some(&feito.unidade),
    )
    .map_err(|_| E::EfeitoIncerto)?;
    let arvore = get(
        envio,
        tela_lida.arvore.as_str(),
        cookie_valor,
        &mut requisicoes,
    )
    .map_err(|_| E::EfeitoIncerto)?;
    let nup =
        crate::arvore_processo::nup_declarado_na_arvore(&arvore.html).ok_or(E::EfeitoIncerto)?;
    crate::arvore_processo::ler_arvore(&arvore.html, &nup, &id).map_err(|_| E::EfeitoIncerto)?;
    feito.confirmado = true;
    feito.nup = Some(nup);
    feito.requisicoes = requisicoes;
    Ok(feito)
}

#[cfg(test)]
#[path = "tests_fluxo.rs"]
mod tests;

#[cfg(test)]
#[path = "tests_hipotese.rs"]
mod tests_hipotese;

#[cfg(test)]
#[path = "acervo.rs"]
mod acervo;
