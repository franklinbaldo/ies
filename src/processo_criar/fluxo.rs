//! Fluxo Web de criação de processo: prévia sem POST criador; escrita única
//! com 302 e verificação da identidade na árvore. Nunca há retry.
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDeSessao, EstadoDoProcesso,
    OperacaoDeFormulario, RespostaDoEnvio, ResultadoDoEnvio, TelaServida, derivar_submissao,
    enviar_submissao_via,
};
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo_criar_cadastro::conferir_cadastro;
use crate::processo_criar_lista::{ErroListaDeTipos, EscolhaDoTipo, tipo_na_lista};
use crate::processo_criar_menu::link_de_iniciar;
use crate::transporte::decodificar_resposta;
use crate::transporte::sessao::CookieDaSessao;
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
    campos_ocultos_reescritos: &["hdnFlagProcedimentoCadastro", "hdnAssuntos"],
    campos_usuario: &["txtDescricao", "rdoNivelAcesso", "selHipoteseLegal"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::ProcessoCriadoNaUnidade,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ErroCriar {
    Transporte,
    UnidadeDiferente,
    Menu,
    Lista,
    Cadastro,
    Formulario,
    TipoSemIdentificador,
    EscolhaInesperada,
    InteressadoNaoResolvido,
    EfeitoIncerto,
}

impl ErroCriar {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::Transporte => "falha_de_transporte_na_criacao",
            Self::UnidadeDiferente => "unidade_da_criacao_divergente",
            Self::Menu => "menu_de_criacao_nao_reconhecido",
            Self::Lista => "lista_de_tipos_nao_reconhecida",
            Self::Cadastro => "cadastro_de_processo_nao_reconhecido",
            Self::Formulario => "formulario_de_criacao_nao_reconhecido",
            Self::TipoSemIdentificador => "tipo_de_processo_sem_identificador",
            Self::EscolhaInesperada => "cadastro_nao_devolvido_apos_escolha",
            Self::InteressadoNaoResolvido => "interessado_de_processo_nao_resolvido",
            Self::EfeitoIncerto => "efeito_incerto_na_criacao",
        }
    }

    pub(crate) fn saida(self) -> u8 {
        if self == Self::EfeitoIncerto { 3 } else { 1 }
    }
}

pub(crate) struct PedidoCriar<'a> {
    pub(crate) tipo: &'a str,
    pub(crate) especificacao: &'a str,
    pub(crate) interessados: &'a [String],
    pub(crate) nivel: &'a str,
    pub(crate) unidade: Option<&'a str>,
    pub(crate) confirmar: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Criacao {
    pub(crate) tipo: String,
    pub(crate) assunto: String,
    pub(crate) nivel: String,
    pub(crate) unidade: String,
    pub(crate) confirmado: bool,
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
    let resposta = envio.get(url, cookie).map_err(|_| ErroCriar::Transporte)?;
    let mime = resposta.mime.clone();
    let html = decodificar_resposta(resposta).map_err(|_| ErroCriar::Transporte)?;
    Ok(Tela {
        url: url.to_owned(),
        html,
        mime,
    })
}

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
    resultado.map_err(|_| {
        if eh_escrita && tentou {
            ErroCriar::EfeitoIncerto
        } else {
            ErroCriar::Transporte
        }
    })
}

fn id_do_link(link: &str) -> Option<&str> {
    let mut ids = link.split('?').nth(1)?.split('&').filter_map(|par| {
        par.split_once('=').and_then(|(nome, valor)| {
            (nome == "id_tipo_procedimento"
                && !valor.is_empty()
                && valor.bytes().all(|b| b.is_ascii_digit()))
            .then_some(valor)
        })
    });
    let primeiro = ids.next()?;
    ids.next().is_none().then_some(primeiro)
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
    let mut acoes = 0;
    for par in partes.consulta.unwrap_or_default().split('&') {
        if par == format!("acao={acao}") {
            acoes += 1;
        }
    }
    if acoes != 1 {
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
    let vezes = partes
        .consulta
        .unwrap_or_default()
        .split('&')
        .filter(|par| *par == format!("acao={acao}"))
        .count();
    if vezes != 1 {
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
    let ids: Vec<_> = consulta
        .split('&')
        .filter_map(|par| par.strip_prefix("id_procedimento="))
        .filter(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()))
        .collect();
    let [id] = ids.as_slice() else { return Err(ErroCriar::EfeitoIncerto); };
    let id = (*id).to_owned();
    Ok((url, id))
}

/// Um POST de escolha é leitura do cadastro. O POST com CADASTRAR só
/// acontece após conferência da lista, do formulário e de todos os campos.
pub(crate) fn criar<V: EnvioDoControle>(
    origem: &str,
    url_controle: &str,
    cookie: &CookieDaSessao,
    pedido: &PedidoCriar<'_>,
    envio: &mut V,
) -> Result<Criacao, ErroCriar> {
    use ErroCriar as E;
    let instalacao = Instalacao::interpretar(origem).ok_or(E::Transporte)?;
    let cookie_valor = cookie.valor().map_err(|_| E::Transporte)?;
    let mut requisicoes = 0;
    let controle = get(envio, url_controle, cookie_valor, &mut requisicoes)?;
    let unidade = crate::controle_reconhecido::unidade_do_controle(&controle.html)
        .map_err(|_| E::UnidadeDiferente)?;
    if pedido.unidade.is_some_and(|exigida| exigida != unidade) {
        return Err(E::UnidadeDiferente);
    }
    let link = link_de_iniciar(&instalacao, &controle.url, &controle.html).map_err(|_| E::Menu)?;
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
                return Err(E::Lista);
            };
            lista.html = html;
            tipo_na_lista(&lista.html, pedido.tipo).map_err(|_| E::Lista)?
        }
        outro => outro.map_err(|_| E::Lista)?,
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
    let observado =
        conferir_cadastro(&cadastro.html, &id_tipo, pedido.nivel).map_err(|_| E::Cadastro)?;
    // A busca de interessados é AJAX específico por instalação. Não enviar
    // uma lista inventada ou silenciosamente vazia.
    if !pedido.interessados.is_empty() {
        return Err(E::InteressadoNaoResolvido);
    }
    let mut feito = Criacao {
        tipo: escolhido.nome,
        assunto: observado.assunto.clone(),
        nivel: pedido.nivel.to_owned(),
        unidade,
        confirmado: false,
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
    // Derive o POST inclusive na prévia: verificar a estrutura antes de
    // prometer que uma confirmação posterior poderia criar o processo.
    let codificacao = codificacao_da_tela(&cadastro.mime).ok_or(E::Formulario)?;
    let servida = TelaServida {
        url: &cadastro.url,
        html: &cadastro.html,
        codificacao,
    };
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
    let tela_do_processo = get(envio, &url, cookie_valor, &mut requisicoes)
        .map_err(|_| E::EfeitoIncerto)?;
    let tela_lida = crate::arvore_processo::ler_tela_do_processo(
        &instalacao, &tela_do_processo.html, &id, Some(&feito.unidade),
    ).map_err(|_| E::EfeitoIncerto)?;
    let arvore = get(envio, tela_lida.arvore.as_str(), cookie_valor, &mut requisicoes)
        .map_err(|_| E::EfeitoIncerto)?;
    let nup = crate::arvore_processo::nup_declarado_na_arvore(&arvore.html)
        .ok_or(E::EfeitoIncerto)?;
    crate::arvore_processo::ler_arvore(&arvore.html, &nup, &id)
        .map_err(|_| E::EfeitoIncerto)?;
    feito.confirmado = true;
    feito.nup = Some(nup);
    feito.requisicoes = requisicoes;
    Ok(feito)
}
