//! Consulta da hipótese legal que o cadastro restrito publica por AJAX.
//! Não executa JavaScript, não inventa opções nem repete requisições.
use crate::formulario_web::{ConsultaAjax, EfeitoAvaliado, TelaServida, derivar_consulta_ajax};
use crate::html_atributos::{atributo_presente, valor_do_atributo};
use crate::instalacao::Instalacao;
use crate::transporte::{
    FalhaTransporte, StatusHttpRecusado, corpo::decode_http_body, ler_limitado,
};
use crate::visao_do_controle::EnvioDoControle;

const PREFIXO: &str = "objAjaxHipoteseLegal = new infraAjaxMontarSelect('selHipoteseLegal','";
const CONSULTA: ConsultaAjax = ConsultaAjax {
    acao_ajax: "hipotese_legal_select_nome_base_legal",
    campos: &[
        "primeiroItemValor",
        "primeiroItemDescricao",
        "valorItemSelecionado",
        "staNivelAcesso",
    ],
    efeito: EfeitoAvaliado::ConsultaSemEfeitoObservado {
        evidencia: "docs/como-o-sei-funciona/inicio-do-processo.md: select AJAX restrito",
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ErroHipotese {
    ConsultaNaoDeclarada,
    ConsultaNaoDerivada,
    RespostaNaoReconhecida,
    OpcaoNaoUnica,
    EscolhaNaoOferecida,
    EscolhaInlineNaoVerificada,
    Transporte {
        subcodigo: &'static str,
        http: Option<StatusHttpRecusado>,
    },
}

impl ErroHipotese {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::ConsultaNaoDeclarada => "hipotese_legal_ajax_nao_declarada",
            Self::ConsultaNaoDerivada => "hipotese_legal_ajax_nao_derivada",
            Self::RespostaNaoReconhecida => "hipotese_legal_ajax_resposta_nao_reconhecida",
            Self::OpcaoNaoUnica => "hipotese_legal_ajax_nao_unica",
            Self::EscolhaNaoOferecida => "hipotese_legal_escolha_nao_oferecida",
            Self::EscolhaInlineNaoVerificada => "hipotese_legal_escolha_inline_nao_verificada",
            Self::Transporte { subcodigo, .. } => subcodigo,
        }
    }

    pub(crate) fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Transporte { http, .. } => http.as_ref(),
            _ => None,
        }
    }

    fn de_transporte(falha: &FalhaTransporte) -> Self {
        Self::Transporte {
            subcodigo: falha.subcodigo(),
            http: falha.status_http().cloned(),
        }
    }
}

/// Reconhece só a declaração literal servida ao select esperado.
/// Se houver duas declarações, nem a primeira nem a última prevalece.
fn url_publicada(html: &str) -> Result<String, ErroHipotese> {
    let urls: Vec<_> = html
        .lines()
        .map(str::trim)
        .filter_map(|linha| linha.strip_prefix(PREFIXO))
        .filter_map(|resto| resto.strip_suffix("');"))
        .collect();
    match urls.as_slice() {
        [url] if url.starts_with("controlador_ajax.php?") => Ok((*url).to_owned()),
        _ => Err(ErroHipotese::ConsultaNaoDeclarada),
    }
}

/// A resposta do infraAjaxMontarSelect é um select XML.
fn opcoes(xml: &str) -> Result<Vec<(String, String)>, ErroHipotese> {
    use ErroHipotese as E;
    let dom =
        tl::parse(xml, tl::ParserOptions::default()).map_err(|_| E::RespostaNaoReconhecida)?;
    let mut selects = dom.query_selector("select").into_iter().flatten();
    let select = selects.next().ok_or(E::RespostaNaoReconhecida)?;
    if selects.next().is_some() {
        return Err(E::RespostaNaoReconhecida);
    }
    let parser = dom.parser();
    let tag = select
        .get(parser)
        .and_then(tl::Node::as_tag)
        .ok_or(E::RespostaNaoReconhecida)?;
    let mut opcoes = Vec::new();
    for handle in tag.query_selector(parser, "option").into_iter().flatten() {
        let opcao = handle
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(E::RespostaNaoReconhecida)?;
        if atributo_presente(opcao, "disabled").map_err(|_| E::RespostaNaoReconhecida)? {
            continue;
        }
        let valor = valor_do_atributo(opcao, "value").map_err(|_| E::RespostaNaoReconhecida)?;
        let Some(id) = valor.filter(|v| !v.is_empty() && v != "null") else {
            continue;
        };
        let texto =
            crate::transporte::texto_de_recusa::decodificar_entidades(&opcao.inner_text(parser));
        if texto.trim().is_empty() {
            return Err(E::RespostaNaoReconhecida);
        }
        opcoes.push((id, texto.trim().to_owned()));
    }
    Ok(opcoes)
}

/// Sem escolha explícita, apenas uma opção efetiva é segura.
pub(crate) fn opcao_unica(xml: &str) -> Result<String, ErroHipotese> {
    let opcoes = opcoes(xml)?;
    match opcoes.as_slice() {
        [(id, _)] => Ok(id.clone()),
        _ => Err(ErroHipotese::OpcaoNaoUnica),
    }
}

/// Seleção por ID e nome exatamente como observados, nunca pela posição.
/// O ID é opaco; recusar duplicata mesmo se as descrições forem diferentes.
pub(crate) fn opcao_selecionada(xml: &str, escolha: Option<&str>) -> Result<String, ErroHipotese> {
    use ErroHipotese as E;
    let Some(escolha) = escolha else {
        return opcao_unica(xml);
    };
    let (id, nome) = escolha.split_once('#').ok_or(E::EscolhaNaoOferecida)?;
    if id.is_empty() || nome.trim().is_empty() || nome != nome.trim() {
        return Err(E::EscolhaNaoOferecida);
    }
    let opcoes = opcoes(xml)?;
    for (indice, (id_lido, _)) in opcoes.iter().enumerate() {
        if opcoes[..indice].iter().any(|(outro, _)| outro == id_lido) {
            return Err(E::OpcaoNaoUnica);
        }
    }
    let mut correspondentes = opcoes
        .iter()
        .filter(|(id_lido, nome_lido)| id_lido == id && nome_lido == nome);
    let primeiro = correspondentes.next().ok_or(E::EscolhaNaoOferecida)?;
    if correspondentes.next().is_some() {
        return Err(E::OpcaoNaoUnica);
    }
    Ok(primeiro.0.clone())
}

/// Um único POST de consulta sem efeito de escrita; nenhuma execução de JS.
/// O ID retornado só pode entrar no cadastro depois da prova de unicidade.
pub(crate) fn resolver<V: EnvioDoControle>(
    envio: &mut V,
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    cookie: &str,
    requisicoes: &mut u32,
    escolha: Option<&str>,
) -> Result<String, ErroHipotese> {
    use ErroHipotese as E;
    let url = url_publicada(tela.html)?;
    let consulta = derivar_consulta_ajax(
        &CONSULTA,
        instalacao,
        tela,
        &url,
        &[
            ("primeiroItemValor", "null"),
            ("primeiroItemDescricao", " "),
            ("valorItemSelecionado", ""),
            ("staNivelAcesso", "1"),
        ],
    )
    .map_err(|_| E::ConsultaNaoDerivada)?;
    *requisicoes += 1;
    let resposta = envio
        .post(consulta, cookie)
        .map_err(|f| E::de_transporte(&f))?;
    if resposta.resposta.status_code != 200 {
        let falha = crate::transporte::recusar_status(resposta.resposta);
        return Err(E::de_transporte(&falha));
    }
    if !matches!(
        resposta
            .resposta
            .mime
            .split(';')
            .next()
            .unwrap_or_default()
            .trim(),
        "application/xml" | "text/xml"
    ) {
        return Err(E::RespostaNaoReconhecida);
    }
    let mime = resposta.resposta.mime.clone();
    let bytes = ler_limitado(resposta.resposta.corpo, crate::MAX_HTML_BYTES)
        .map_err(|f| E::de_transporte(&f))?;
    let xml = decode_http_body(&bytes, &mime).map_err(|_| E::RespostaNaoReconhecida)?;
    opcao_selecionada(&xml, escolha)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exige_uma_opcao_efetiva_depois_do_placeholder() {
        let xml = "<?xml version='1.0'?><select><option value='null'> </option><option value='42'>Unica</option></select>";
        assert_eq!(opcao_unica(xml), Ok("42".to_owned()));
        assert_eq!(
            opcao_unica("<?xml version='1.0'?><select><option value='null'> </option></select>"),
            Err(ErroHipotese::OpcaoNaoUnica)
        );
    }

    #[test]
    fn red_escolhe_segunda_hipotese_por_id_e_nome_exatos() {
        // Cenário de opções AJAX múltiplas observado no acervo UnB, #245.
        let xml = "<select><option value='null'> </option><option value='31'>Base A</option><option value='32'>Base B</option></select>";
        assert_eq!(
            opcao_selecionada(xml, Some("32#Base B")),
            Ok("32".to_owned())
        );
    }

    #[test]
    fn red_escolha_nao_publicada_eh_recusada_mesmo_com_opcao_unica() {
        let xml = "<select><option value='31'>Base A</option></select>";
        assert_eq!(
            opcao_selecionada(xml, Some("31#Outra base")),
            Err(ErroHipotese::EscolhaNaoOferecida)
        );
    }

    #[test]
    fn opcao_desabilitada_ou_id_duplicado_nao_eh_aceita() {
        let desabilitada = "<select><option value='31' disabled>Base A</option><option value='32'>Base B</option></select>";
        assert_eq!(
            opcao_selecionada(desabilitada, Some("31#Base A")),
            Err(ErroHipotese::EscolhaNaoOferecida)
        );
        let duplicada = "<select><option value='32'>Base B</option><option value='32'>Outra B</option></select>";
        assert_eq!(
            opcao_selecionada(duplicada, Some("32#Base B")),
            Err(ErroHipotese::OpcaoNaoUnica)
        );
    }

    #[test]
    fn recusa_dupla_declaracao_sem_escolher_url() {
        let linha = format!(
            "{PREFIXO}controlador_ajax.php?acao_ajax=hipotese_legal_select_nome_base_legal');"
        );
        assert_eq!(
            url_publicada(&format!("{linha}\n{linha}")),
            Err(ErroHipotese::ConsultaNaoDeclarada)
        );
    }
}
