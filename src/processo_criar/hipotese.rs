//! Consulta da hipótese legal que o cadastro restrito publica por AJAX.
//! Não executa JavaScript, não inventa opções nem repete requisições.
use crate::formulario_web::{ConsultaAjax, EfeitoAvaliado, TelaServida, derivar_consulta_ajax};
use crate::html_atributos::{atributo_presente, valor_do_atributo};
use crate::instalacao::Instalacao;
#[cfg(test)]
use crate::transporte::StatusHttpRecusado;
use crate::transporte::{FalhaTransporte, corpo::decode_http_body, ler_limitado};
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

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroHipotese {
    // Formato de tela não é categoria de domínio: nome da fronteira HTML
    // mantém o subcódigo público, enquanto a causa é tipada por Problema.
    Html(#[from] crate::erro_html::ErroHtml),
    OpcaoNaoUnica,
    OpcoesDisponiveis(Vec<(String, String)>),
    EscolhaNaoOferecida,
    EscolhaInlineNaoVerificada,
    // A causa concreta sobe, em vez de extrair nome de texto e
    // reconstruir um snapshot que perderia a variante original.
    Transporte(#[from] FalhaTransporte),
}

impl ErroHipotese {
    fn consulta_nao_declarada() -> Self {
        Self::Html(crate::erro_html::ErroHtml::ausente(
            "hipotese_legal_ajax_nao_declarada",
        ))
    }

    fn consulta_nao_derivada() -> Self {
        Self::Html(crate::erro_html::ErroHtml::valor(
            "hipotese_legal_ajax_nao_derivada",
        ))
    }

    fn resposta_nao_reconhecida() -> Self {
        Self::Html(crate::erro_html::ErroHtml::html(
            "hipotese_legal_ajax_resposta_nao_reconhecida",
        ))
    }

    /// Somente opções distintas, habilitadas e imprimíveis realmente vistas no
    /// AJAX podem ser oferecidas para uma próxima prévia com ID#NOME.
    #[cfg(test)]
    pub(crate) fn opcoes_para_escolha(&self) -> Option<&[(String, String)]> {
        match self {
            Self::OpcoesDisponiveis(opcoes) => Some(opcoes),
            _ => None,
        }
    }

    #[cfg(test)]
    pub(crate) fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Transporte(falha) => falha.status_http(),
            Self::Html(_)
            | Self::OpcaoNaoUnica
            | Self::OpcoesDisponiveis(_)
            | Self::EscolhaNaoOferecida
            | Self::EscolhaInlineNaoVerificada => None,
        }
    }

    fn de_transporte(falha: &FalhaTransporte) -> Self {
        Self::Transporte(falha.clone())
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
        _ => Err(ErroHipotese::consulta_nao_declarada()),
    }
}

/// A resposta do infraAjaxMontarSelect é um select XML.
fn opcoes(xml: &str) -> Result<Vec<(String, String)>, ErroHipotese> {
    use ErroHipotese as E;
    let dom =
        tl::parse(xml, tl::ParserOptions::default()).map_err(|_| E::resposta_nao_reconhecida())?;
    let mut selects = dom.query_selector("select").into_iter().flatten();
    let select = selects.next().ok_or(E::resposta_nao_reconhecida())?;
    if selects.next().is_some() {
        return Err(E::resposta_nao_reconhecida());
    }
    let parser = dom.parser();
    let tag = select
        .get(parser)
        .and_then(tl::Node::as_tag)
        .ok_or(E::resposta_nao_reconhecida())?;
    let mut opcoes = Vec::new();
    for handle in tag.query_selector(parser, "option").into_iter().flatten() {
        let opcao = handle
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(E::resposta_nao_reconhecida())?;
        if atributo_presente(opcao, "disabled").map_err(|_| E::resposta_nao_reconhecida())? {
            continue;
        }
        let valor = valor_do_atributo(opcao, "value").map_err(|_| E::resposta_nao_reconhecida())?;
        let Some(id) = valor.filter(|v| !v.is_empty() && v != "null") else {
            continue;
        };
        let texto = html_escape::decode_html_entities(&opcao.inner_text(parser)).into_owned();
        if texto.trim().is_empty() {
            return Err(E::resposta_nao_reconhecida());
        }
        opcoes.push((id, texto.trim().to_owned()));
    }
    Ok(opcoes)
}

/// O XML do servidor é dado não confiável. Nomes e IDs oferecidos como
/// orientação não podem transportar controles nem URLs/credenciais.
fn publicavel(valor: &str) -> bool {
    if valor.chars().any(char::is_control) {
        return false;
    }
    let minusculo = valor.to_ascii_lowercase();
    let fragmentos_sensiveis = [
        "https://",
        "http://",
        "infra_hash",
        "cookie=",
        "senha=",
        "controlador.php?",
        "controlador_ajax.php?",
    ];
    !fragmentos_sensiveis
        .iter()
        .any(|fragmento| minusculo.contains(*fragmento))
}

/// Sem escolha explícita, apenas uma opção efetiva é segura.
pub(crate) fn opcao_unica(xml: &str) -> Result<String, ErroHipotese> {
    let opcoes = opcoes(xml)?;
    match opcoes.as_slice() {
        // Mesmo uma opção única pode ter vindo contaminada do XML remoto.
        // Não publicar seu ID nem enviá-lo no formulário de criação.
        [(id, nome)] if id.contains('#') || !publicavel(id) || !publicavel(nome) => {
            Err(ErroHipotese::resposta_nao_reconhecida())
        }
        [(id, _)] => Ok(id.clone()),
        [] => Err(ErroHipotese::OpcaoNaoUnica),
        // Não oferecer valores que a própria CLI não consegue representar
        // sem ambiguidade ou que possam controlar a saída humana.
        _ if opcoes
            .iter()
            .any(|(id, nome)| id.contains('#') || !publicavel(id) || !publicavel(nome)) =>
        {
            Err(ErroHipotese::OpcaoNaoUnica)
        }
        _ if opcoes.iter().enumerate().any(|(indice, (id, _))| {
            opcoes[..indice].iter().any(|(anterior, _)| anterior == id)
        }) =>
        {
            Err(ErroHipotese::OpcaoNaoUnica)
        }
        _ => Err(ErroHipotese::OpcoesDisponiveis(opcoes)),
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
    // Uma escolha explícita não torna confiáveis dados arbitrários
    // de uma resposta XML: recusar sem oferecer ou publicar esse valor.
    if opcoes.iter().any(|(id_lido, nome_lido)| {
        id_lido.contains('#') || !publicavel(id_lido) || !publicavel(nome_lido)
    }) {
        return Err(E::resposta_nao_reconhecida());
    }
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
    .map_err(|_| E::consulta_nao_derivada())?;
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
        return Err(E::resposta_nao_reconhecida());
    }
    let mime = resposta.resposta.mime.clone();
    let bytes = ler_limitado(resposta.resposta.corpo, crate::MAX_HTML_BYTES)
        .map_err(|f| E::de_transporte(&f))?;
    let xml = decode_http_body(&bytes, &mime).map_err(|_| E::resposta_nao_reconhecida())?;
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
    fn red_oferece_id_e_nome_exatos_de_duas_hipoteses_ajax() {
        let xml = "<select><option value='null'>Selecione</option><option value='31'>Base A</option><option value='32'>Base B</option></select>";
        let erro = opcao_selecionada(xml, None).expect_err("sem escolha nao escreve");
        assert_eq!(
            erro.opcoes_para_escolha(),
            Some(
                &[
                    ("31".to_owned(), "Base A".to_owned()),
                    ("32".to_owned(), "Base B".to_owned()),
                ][..],
            )
        );
    }

    #[test]
    fn red_nao_sugere_hipotese_desabilitada() {
        let xml = "<select><option value='31'>Base A</option><option value='32' disabled>Desabilitada</option><option value='33'>Base C</option></select>";
        let erro = opcao_selecionada(xml, None).expect_err("multiplas hipoteses");
        assert_eq!(
            erro.opcoes_para_escolha(),
            Some(
                &[
                    ("31".to_owned(), "Base A".to_owned()),
                    ("33".to_owned(), "Base C".to_owned()),
                ][..],
            )
        );
    }

    #[test]
    fn ids_duplicados_ou_texto_com_controle_nao_sao_sugeridos() {
        let duplicados =
            "<select><option value='31'>A</option><option value='31'>B</option></select>";
        let erro = opcao_unica(duplicados).expect_err("id repetido nao e selecionavel");
        assert_eq!(erro, ErroHipotese::OpcaoNaoUnica);
        assert_eq!(erro.opcoes_para_escolha(), None);

        let com_controle = "<select><option value='31'>Base A</option><option value='32'>Base\u{001b} B</option></select>";
        let erro = opcao_unica(com_controle).expect_err("controle nao pode ir ao terminal");
        assert_eq!(erro, ErroHipotese::OpcaoNaoUnica);
        assert_eq!(erro.opcoes_para_escolha(), None);
    }

    #[test]
    fn url_em_nome_de_hipotese_nao_eh_exibida_como_sugestao() {
        let xml = "<select><option value='31'>Base A</option><option value='32'>https://example.invalid/segredo</option></select>";
        let erro = opcao_unica(xml).expect_err("url nao eh nome publicavel");
        assert_eq!(erro, ErroHipotese::OpcaoNaoUnica);
        assert_eq!(erro.opcoes_para_escolha(), None);
    }

    #[test]
    fn red_opcao_unica_com_url_no_id_nao_pode_ser_enviada() {
        // Resposta sintética adversarial. O valor XML é dado não confiável,
        // mesmo quando só existe uma hipótese habilitada.
        let xml =
            "<select><option value='https://example.invalid/marcador'>Base A</option></select>";
        assert_eq!(
            opcao_unica(xml),
            Err(ErroHipotese::resposta_nao_reconhecida())
        );
        assert_eq!(
            opcao_selecionada(xml, Some("https://example.invalid/marcador#Base A")),
            Err(ErroHipotese::resposta_nao_reconhecida())
        );
    }

    #[test]
    fn red_escolha_explicita_nao_autoriza_nome_com_url() {
        // Uma opção maliciosa não pode ser transformada em argumento de
        // confirmação só porque ID#NOME coincide com o XML.
        let xml = "<select><option value='31'>https://example.invalid/marcador</option></select>";
        assert_eq!(
            opcao_selecionada(xml, Some("31#https://example.invalid/marcador")),
            Err(ErroHipotese::resposta_nao_reconhecida())
        );
        assert_eq!(
            opcao_unica(xml),
            Err(ErroHipotese::resposta_nao_reconhecida())
        );
    }

    #[test]
    fn escolha_segura_nao_aproveita_lista_contaminada() {
        let xml = "<select><option value='31'>Base A</option><option value='32'>https://example.invalid/marcador</option></select>";
        assert_eq!(
            opcao_selecionada(xml, Some("31#Base A")),
            Err(ErroHipotese::resposta_nao_reconhecida())
        );
    }

    #[test]
    fn recusa_dupla_declaracao_sem_escolher_url() {
        let linha = format!(
            "{PREFIXO}controlador_ajax.php?acao_ajax=hipotese_legal_select_nome_base_legal');"
        );
        assert_eq!(
            url_publicada(&format!("{linha}\n{linha}")),
            Err(ErroHipotese::consulta_nao_declarada())
        );
    }
    #[test]
    fn causa_ajax_preserva_folhas_e_tipo_estrutural() {
        let _ausente = url_publicada("<html></html>").expect_err("URL não publicada");
        let transporte = ErroHipotese::de_transporte(&FalhaTransporte::Rede);
        assert!(matches!(
            &transporte,
            ErroHipotese::Transporte(FalhaTransporte::Rede)
        ));
    }

    #[test]
    fn causa_ajax_propagada_mantem_http_saneado() {
        let falha = FalhaTransporte::Status(StatusHttpRecusado::sem_mensagem(503));
        let erro = ErroHipotese::de_transporte(&falha);
        assert_eq!(erro.status_http().map(|http| http.status), Some(503));
    }
}
