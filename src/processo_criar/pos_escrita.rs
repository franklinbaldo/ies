//! Reconciliação do POST criador #245 pela caixa Gerados e propriedades
//! independentes. Não fabrica links, não repete requisições e nunca executa JS.
use super::erro_fluxo::CausaRespostaCriadora;
use super::{Criacao, ErroCriar, get, reconciliacao, retorno_da_criacao};
use crate::formulario_web::{ErroSubmissaoFormulario, decodificar_atributo};
use crate::html_atributos::{nos_com_id, valor_do_atributo};
use crate::instalacao::Instalacao;
use crate::visao_do_controle::EnvioDoControle;
use reconciliacao::desfechos::{Desfecho, ProcessoConferido, classificar};

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(super) enum CausaPosEscrita {
    Transporte(#[from] ErroSubmissaoFormulario),
    Resposta(#[from] CausaRespostaCriadora),
}

impl CausaPosEscrita {
    fn inconclusiva(
        self,
        posterior: Result<Vec<reconciliacao::CandidatoGerado>, ErroCriar>,
        correspondentes: Option<Vec<String>>,
        observados_antes_da_falha: Option<Box<[reconciliacao::CandidatoGerado]>>,
    ) -> ErroCriar {
        let posterior = posterior.map_err(Box::new);
        match self {
            Self::Transporte(causa_post) => ErroCriar::PosEscrita {
                causa_post,
                posterior,
                correspondentes,
                observados_antes_da_falha,
            },
            Self::Resposta(causa) => ErroCriar::PosRespostaInesperada {
                causa,
                posterior,
                correspondentes,
                observados_antes_da_falha,
            },
        }
    }
}

/// Aceita o href servido na linha da tabela Gerados cujo par (ID, NUP)
/// coincide integralmente com a fotografia. Duplicidades falham fechadas.
fn link_do_gerado(
    html: &str,
    candidato: &reconciliacao::CandidatoGerado,
) -> Result<String, ErroCriar> {
    use reconciliacao::ErroFotografia as F;
    let erro = || ErroCriar::Fotografia(F::LinkDoCandidatoNaoComprovado);
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| erro())?;
    let parser = dom.parser();
    let tabelas = nos_com_id(&dom, "tblProcessosGerados").map_err(|_| erro())?;
    let [tabela] = tabelas.as_slice() else {
        return Err(erro());
    };
    let tabela = tabela
        .get(parser)
        .and_then(tl::Node::as_tag)
        .ok_or_else(erro)?;
    let alvo = format!("P{}", candidato.id);
    let mut encontrado = None;
    for no in tabela.query_selector(parser, "tr").into_iter().flatten() {
        let Some(linha) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let id = valor_do_atributo(linha, "id").map_err(|_| erro())?;
        if id.as_deref() != Some(alvo.as_str()) {
            continue;
        }
        for link in linha.query_selector(parser, "a").into_iter().flatten() {
            let Some(ancora) = link.get(parser).and_then(tl::Node::as_tag) else {
                continue;
            };
            if ancora.inner_text(parser).trim() != candidato.nup {
                continue;
            }
            let href = valor_do_atributo(ancora, "href")
                .map_err(|_| erro())?
                .ok_or_else(erro)?;
            let href = decodificar_atributo(&href).ok_or_else(erro)?;
            if encontrado.replace(href).is_some() {
                return Err(ErroCriar::Fotografia(F::LinksDoCandidatoRepetidos));
            }
        }
    }
    encontrado.ok_or_else(erro)
}

#[allow(clippy::too_many_arguments)] // contexto e identidade explícitos
fn conferir_candidato<V: EnvioDoControle>(
    envio: &mut V,
    instalacao: &Instalacao,
    url_controle: &str,
    cookie: &str,
    unidade: &str,
    html_controle: &str,
    candidato: &reconciliacao::CandidatoGerado,
    requisicoes: &mut u32,
) -> Result<ProcessoConferido, ErroCriar> {
    let href = link_do_gerado(html_controle, candidato)?;
    let (url, id) = retorno_da_criacao(instalacao, &href, url_controle).map_err(|_| {
        ErroCriar::Fotografia(reconciliacao::ErroFotografia::LinkDoCandidatoNaoComprovado)
    })?;
    if id != candidato.id {
        return Err(ErroCriar::Fotografia(
            reconciliacao::ErroFotografia::LinkDoCandidatoNaoComprovado,
        ));
    }
    let pagina = get(envio, &url, cookie, requisicoes)?;
    let tela =
        crate::arvore_processo::ler_tela_do_processo(instalacao, &pagina.html, &id, Some(unidade))
            .map_err(ErroCriar::Tela)?;
    let pagina_arvore = get(envio, tela.arvore.as_str(), cookie, requisicoes)?;
    crate::arvore_processo::ler_arvore(&pagina_arvore.html, &candidato.nup, &id)
        .map_err(ErroCriar::Arvore)?;
    let endereco =
        crate::arvore_processo::ler_link_das_propriedades(instalacao, &pagina_arvore.html, &id)
            .map_err(ErroCriar::Propriedades)?
            .ok_or(ErroCriar::PropriedadesNaoOferecidas)?;
    let pagina_propriedades = get(envio, endereco.as_str(), cookie, requisicoes)?;
    let propriedades = crate::propriedades_processo::ler_propriedades(
        &pagina_propriedades.html,
        &candidato.nup,
        &id,
        endereco.fonte,
    )
    .map_err(ErroCriar::Propriedades)?;
    Ok(ProcessoConferido {
        id: candidato.id.clone(),
        nup: candidato.nup.clone(),
        tipo: propriedades.tipo,
        especificacao: Some(propriedades.especificacao),
        nivel: Some(propriedades.nivel_de_acesso.como_texto().to_owned()),
        interessados: propriedades.interessados,
    })
}

/// Um GET do Controle e três GET por candidato (tela, árvore, propriedades).
/// A saída aplicada só ocorre depois de todas as identidades conferidas.
#[allow(clippy::too_many_arguments)]
pub(super) fn reconciliar<V: EnvioDoControle>(
    envio: &mut V,
    instalacao: &Instalacao,
    url_controle: &str,
    cookie: &str,
    feito: &mut Criacao,
    antes: Option<&reconciliacao::FotografiaDosGerados>,
    especificacao: &str,
    requisicoes: &mut u32,
    causa: CausaPosEscrita,
) -> Result<(), ErroCriar> {
    // Sem fotografia anterior completa não há base de comparação: efeito
    // incerto sem nova leitura.
    let Some(antes) = antes else {
        return Err(causa.inconclusiva(
            Err(ErroCriar::Fotografia(
                reconciliacao::ErroFotografia::CoberturaIncompleta,
            )),
            None,
            None,
        ));
    };
    let mut observados_antes_da_falha = None;
    // Provas positivas obtidas antes de uma falha posterior continuam
    // observadas, mas NÃO permitem concluir unicidade nem autoria.
    let mut correspondentes_parciais = Vec::new();
    let posterior = (|| {
        let controle = get(envio, url_controle, cookie, requisicoes)?;
        let depois = reconciliacao::FotografiaDosGerados::da_tela(&controle.html, &feito.unidade)
            .map_err(ErroCriar::Fotografia)?;
        let candidatos = antes.novos(&depois).map_err(ErroCriar::Fotografia)?;
        // A fotografia já observou estes candidatos; uma falha posterior
        // de conferência não torna os NUPs não observados no diagnóstico.
        observados_antes_da_falha = Some(candidatos.clone().into_boxed_slice());
        let mut provas = Vec::new();
        for candidato in &candidatos {
            let conferido = conferir_candidato(
                envio,
                instalacao,
                url_controle,
                cookie,
                &feito.unidade,
                &controle.html,
                candidato,
                requisicoes,
            )?;
            if conferido.tipo.as_deref() == Some(feito.tipo.as_str())
                && conferido.especificacao.as_deref() == Some(especificacao)
                && conferido.nivel.as_deref() == Some(feito.nivel.as_str())
                && reconciliacao::desfechos::interessados_conferem(
                    &feito.interessados,
                    &conferido.interessados,
                )
            {
                correspondentes_parciais.push(conferido.nup.clone());
            }
            provas.push(conferido);
        }
        let desfecho = classificar(
            &candidatos,
            &provas,
            &feito.tipo,
            especificacao,
            &feito.nivel,
            &feito.interessados,
        )
        .map_err(ErroCriar::Conferencia)?;
        Ok::<_, ErroCriar>((candidatos, desfecho))
    })();
    match posterior {
        // A mesma conta pode ter recebido criacao identica de outro
        // agente entre as duas fotografias. Propriedades iguais atestam
        // apenas coincidencia, nunca autoria de um POST sem resposta.
        Ok((candidatos, Desfecho::CoincidenciaUnica { nup })) => {
            Err(causa.inconclusiva(Ok(candidatos), Some(vec![nup]), None))
        }
        Ok((candidatos, Desfecho::CandidatosAmbiguos { nups })) => {
            Err(causa.inconclusiva(Ok(candidatos), Some(nups), None))
        }
        Ok((candidatos, _)) => Err(causa.inconclusiva(Ok(candidatos), None, None)),
        Err(erro) => Err(causa.inconclusiva(
            Err(erro),
            (!correspondentes_parciais.is_empty()).then_some(correspondentes_parciais),
            observados_antes_da_falha,
        )),
    }
}
