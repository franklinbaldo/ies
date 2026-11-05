//! Núcleo offline da criação de documento interno (#246). Nenhuma rede.
mod fluxo;
pub(crate) use fluxo::{ErroPreviaCriacao, PedidoCriacao, PreviaCriacao, obter_previa};

#[cfg(test)]
use crate::arvore_processo::DocumentoNaArvore;
use crate::arvore_processo::endereco::conferir_endereco_relativo;
use crate::instalacao::Instalacao;
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::{nos_com_id, valor_do_atributo};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroCriar {
    ListaForaDaForma,
    TipoAusente,
    TipoAmbiguo,
    #[cfg(test)]
    NaoAplicado,
    #[cfg(test)]
    EfeitoIncerto,
    LinkDoTipoNaoNavegavel,
    EnderecoDoTipoNaoReconhecido,
    CadastroForaDaForma,
    TipoDoCadastroDiferente,
    DescricaoNaoOferecida,
    NivelNaoOferecido,
}
impl ErroCriar {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::ListaForaDaForma => "lista_de_tipos_fora_da_forma_observada",
            Self::TipoAusente => "tipo_de_documento_nao_oferecido",
            Self::TipoAmbiguo => "tipo_de_documento_ambiguo",
            #[cfg(test)]
            Self::NaoAplicado => "documento_criado_nao_comprovado",
            #[cfg(test)]
            Self::EfeitoIncerto => "efeito_incerto_na_criacao_de_documento",
            Self::LinkDoTipoNaoNavegavel => "link_do_tipo_de_documento_nao_navegavel",
            Self::EnderecoDoTipoNaoReconhecido => "endereco_do_tipo_de_documento_nao_reconhecido",
            Self::CadastroForaDaForma => "cadastro_de_documento_fora_da_forma_observada",
            Self::TipoDoCadastroDiferente => "tipo_do_cadastro_diferente_do_solicitado",
            Self::DescricaoNaoOferecida => "descricao_nao_oferecida_no_cadastro",
            Self::NivelNaoOferecido => "nivel_nao_oferecido_no_cadastro",
        }
    }
}
/// Apenas nomes publicados em table#tblSeries; nunca interpreta JS como código.
pub(crate) fn tipos_oferecidos(html: &str) -> Result<Vec<String>, ErroCriar> {
    let err = ErroCriar::ListaForaDaForma;
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| err)?;
    let tabelas = nos_com_id(&dom, "tblSeries").map_err(|_| err)?;
    let [tabela] = tabelas.as_slice() else {
        return Err(err);
    };
    let parser = dom.parser();
    let tabela = tabela.get(parser).and_then(tl::Node::as_tag).ok_or(err)?;
    if !tabela.name().as_utf8_str().eq_ignore_ascii_case("table") {
        return Err(err);
    }
    let mut tipos = Vec::new();
    for tag in tabela
        .children()
        .all(parser)
        .iter()
        .filter_map(tl::Node::as_tag)
    {
        if !tag.name().as_utf8_str().eq_ignore_ascii_case("a") {
            continue;
        }
        let href = valor_do_atributo(tag, "href").map_err(|_| err)?;
        if href.is_none() {
            continue;
        }
        let onclick = valor_do_atributo(tag, "onclick").map_err(|_| err)?;
        // Na 4.0 "Externo" usa documento_receber, e não documento_gerar.
        // Nas 4.1/5.0, escolher(-1) também é externo.
        let href = decodificar_atributo(href.as_deref().unwrap_or_default()).ok_or(err)?;
        let nativo =
            href.split('?').nth(1).is_some_and(|consulta| {
                consulta.split('&').any(|par| par == "acao=documento_gerar")
            }) || (href == "javascript:void(0);"
                && onclick.as_deref().is_some_and(|chamada| {
                    let chamada = chamada.trim().trim_end_matches(';');
                    chamada
                        .strip_prefix("escolher(")
                        .and_then(|v| v.strip_suffix(')'))
                        .is_some_and(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()))
                }));
        if !nativo {
            continue;
        }
        let nome = tag
            .inner_text(parser)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let nome = decodificar_atributo(&nome).ok_or(err)?;
        if !nome.is_empty() {
            tipos.push(nome);
        }
    }
    Ok(tipos)
}
pub(crate) fn conferir_tipo(tipo: &str, oferecidos: &[String]) -> Result<(), ErroCriar> {
    match oferecidos.iter().filter(|s| s.as_str() == tipo).count() {
        0 => Err(ErroCriar::TipoAusente),
        1 => Ok(()),
        _ => Err(ErroCriar::TipoAmbiguo),
    }
}
/// Sucesso apenas com um id novo, número SEI e rótulo esperados, mantendo
/// exatamente todos os documentos anteriores. Falha pós-POST não é retry.
#[cfg(test)]
pub(crate) fn conferir_criacao(
    antes: &[DocumentoNaArvore],
    depois: &[DocumentoNaArvore],
    tipo: &str,
) -> Result<String, ErroCriar> {
    use ErroCriar as E;
    let mut ids = std::collections::HashSet::new();
    for doc in depois {
        if !ids.insert(doc.id.as_str()) {
            return Err(E::EfeitoIncerto);
        }
    }
    for original in antes {
        if !depois.iter().any(|doc| doc == original) {
            return Err(E::EfeitoIncerto);
        }
    }
    let novos: Vec<_> = depois
        .iter()
        .filter(|doc| !antes.iter().any(|d| d.id == doc.id))
        .collect();
    let novo = match novos.as_slice() {
        [] => return Err(E::NaoAplicado),
        [doc] => *doc,
        _ => return Err(E::EfeitoIncerto),
    };
    let numero = novo
        .numero_sei
        .as_deref()
        .filter(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
        .ok_or(E::EfeitoIncerto)?;
    if novo.rotulo != format!("{tipo} {numero}")
        || antes
            .iter()
            .any(|d| d.numero_sei.as_deref() == Some(numero))
        || depois
            .iter()
            .filter(|d| d.numero_sei.as_deref() == Some(numero))
            .count()
            != 1
    {
        return Err(E::EfeitoIncerto);
    }
    Ok(numero.to_owned())
}


/// Aceita só o href direto e assinado publicado para o tipo exato.
/// Na forma escolher(id), a navegação depende da tela e ainda não foi derivada:
/// falha fechado, sem construir uma URL e sem executar JavaScript.
pub(crate) fn link_direto_do_tipo(
    instalacao: &Instalacao,
    html: &str,
    id_procedimento: &str,
    tipo: &str,
) -> Result<String, ErroCriar> {
    use ErroCriar as E;
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| E::ListaForaDaForma)?;
    let tabelas = nos_com_id(&dom, "tblSeries").map_err(|_| E::ListaForaDaForma)?;
    let [tabela] = tabelas.as_slice() else {
        return Err(E::ListaForaDaForma);
    };
    let tabela = tabela.get(dom.parser()).and_then(tl::Node::as_tag).ok_or(E::ListaForaDaForma)?;
    if !tabela.name().as_utf8_str().eq_ignore_ascii_case("table") {
        return Err(E::ListaForaDaForma);
    }
    let mut links = Vec::new();
    for tag in tabela.children().all(dom.parser()).iter().filter_map(tl::Node::as_tag) {
        if !tag.name().as_utf8_str().eq_ignore_ascii_case("a") {
            continue;
        }
        let nome = tag.inner_text(dom.parser()).split_whitespace().collect::<Vec<_>>().join(" ");
        let nome = decodificar_atributo(&nome).ok_or(E::ListaForaDaForma)?;
        if nome != tipo {
            continue;
        }
        let href = valor_do_atributo(tag, "href").map_err(|_| E::ListaForaDaForma)?
            .ok_or(E::LinkDoTipoNaoNavegavel)?;
        let href = decodificar_atributo(&href).ok_or(E::ListaForaDaForma)?;
        links.push(href);
    }
    let link = match links.as_slice() {
        [] => return Err(E::TipoAusente),
        [link] => link,
        _ => return Err(E::TipoAmbiguo),
    };
    if link.starts_with("javascript:") {
        return Err(E::LinkDoTipoNaoNavegavel);
    }
    conferir_endereco_relativo(instalacao, link, "documento_gerar", id_procedimento)
        .map_err(|_| E::EnderecoDoTipoNaoReconhecido)
}

/// Verifica a identidade do cadastro antes de qualquer POST de criação.
pub(crate) fn conferir_cadastro(
    html: &str,
    tipo: &str,
    descricao: Option<&str>,
    nivel: Option<&str>,
) -> Result<(), ErroCriar> {
    use ErroCriar as E;
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| E::CadastroForaDaForma)?;
    let formularios = nos_com_id(&dom, "frmDocumentoCadastro").map_err(|_| E::CadastroForaDaForma)?;
    let [formulario] = formularios.as_slice() else {
        return Err(E::CadastroForaDaForma);
    };
    let formulario = formulario.get(dom.parser()).and_then(tl::Node::as_tag).ok_or(E::CadastroForaDaForma)?;
    if !formulario.name().as_utf8_str().eq_ignore_ascii_case("form")
        || valor_do_atributo(formulario, "method").map_err(|_| E::CadastroForaDaForma)?
            .as_deref() != Some("post") {
        return Err(E::CadastroForaDaForma);
    }
    let rotulos = nos_com_id(&dom, "lblSerieTitulo").map_err(|_| E::CadastroForaDaForma)?;
    let [rotulo] = rotulos.as_slice() else {
        return Err(E::CadastroForaDaForma);
    };
    let rotulo = rotulo.get(dom.parser()).and_then(tl::Node::as_tag).ok_or(E::CadastroForaDaForma)?;
    if !rotulo.name().as_utf8_str().eq_ignore_ascii_case("label") {
        return Err(E::CadastroForaDaForma);
    }
    let lido = rotulo.inner_text(dom.parser()).split_whitespace().collect::<Vec<_>>().join(" ");
    let lido = decodificar_atributo(&lido).ok_or(E::CadastroForaDaForma)?;
    if lido != tipo {
        return Err(E::TipoDoCadastroDiferente);
    }
    if descricao.is_some_and(|valor| !valor.is_empty()) {
        let controles = nos_com_id(&dom, "txtDescricao").map_err(|_| E::CadastroForaDaForma)?;
        let [controle] = controles.as_slice() else {
            return Err(E::DescricaoNaoOferecida);
        };
        let controle = controle.get(dom.parser()).and_then(tl::Node::as_tag).ok_or(E::DescricaoNaoOferecida)?;
        if !controle.name().as_utf8_str().eq_ignore_ascii_case("input")
            || valor_do_atributo(controle, "name").map_err(|_| E::CadastroForaDaForma)?
                .as_deref() != Some("txtDescricao")
            || valor_do_atributo(controle, "disabled").map_err(|_| E::CadastroForaDaForma)?.is_some()
        {
            return Err(E::DescricaoNaoOferecida);
        }
    }
    if let Some(nivel) = nivel {
        let id = match nivel {
            "publico" => "optPublico",
            "restrito" => "optRestrito",
            "sigiloso" => "optSigiloso",
            _ => return Err(E::NivelNaoOferecido),
        };
        let controles = nos_com_id(&dom, id).map_err(|_| E::CadastroForaDaForma)?;
        let [controle] = controles.as_slice() else {
            return Err(E::NivelNaoOferecido);
        };
        let controle = controle.get(dom.parser()).and_then(tl::Node::as_tag).ok_or(E::NivelNaoOferecido)?;
        if !controle.name().as_utf8_str().eq_ignore_ascii_case("input")
            || valor_do_atributo(controle, "name").map_err(|_| E::CadastroForaDaForma)?
                .as_deref() != Some("rdoNivelAcesso")
            || valor_do_atributo(controle, "type").map_err(|_| E::CadastroForaDaForma)?
                .as_deref() != Some("radio")
            || valor_do_atributo(controle, "disabled").map_err(|_| E::CadastroForaDaForma)?.is_some()
        {
            return Err(E::NivelNaoOferecido);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arvore_processo::NaturezaDoDocumento;
    fn doc(id: &str, numero: Option<&str>, rotulo: &str) -> DocumentoNaArvore {
        DocumentoNaArvore {
            id: id.into(),
            rotulo: rotulo.into(),
            numero_sei: numero.map(str::to_owned),
            pasta: None,
            natureza: NaturezaDoDocumento {
                icone: None,
                cancelado: None,
                motivo_do_cancelamento: None,
                dica: None,
            },
        }
    }
    #[test]
    fn subcodigos_sao_estaveis() {
        assert_eq!(
            ErroCriar::ListaForaDaForma.subcodigo(),
            "lista_de_tipos_fora_da_forma_observada"
        );
        assert_eq!(
            ErroCriar::TipoAusente.subcodigo(),
            "tipo_de_documento_nao_oferecido"
        );
        assert_eq!(
            ErroCriar::TipoAmbiguo.subcodigo(),
            "tipo_de_documento_ambiguo"
        );
        assert_eq!(
            ErroCriar::NaoAplicado.subcodigo(),
            "documento_criado_nao_comprovado"
        );
        assert_eq!(
            ErroCriar::EfeitoIncerto.subcodigo(),
            "efeito_incerto_na_criacao_de_documento"
        );
    }
    #[test]
    fn lista_tipo_ausente_ambiguo_e_externo() {
        // Caso de Todos src/todos/documentos.py::_tipos_da_tela @323eb01
        let html = r#"<table id="tblSeries"><tr><td>
            <a href="controlador.php?acao=documento_gerar">Despacho</a>
            <a href="controlador.php?acao=documento_receber">Externo</a>
            <a href="javascript:void(0);" onclick="escolher(3);">Ofício</a>
            </td></tr></table>"#;
        let tipos = tipos_oferecidos(html).unwrap();
        assert_eq!(tipos, ["Despacho", "Ofício"]);
        assert_eq!(conferir_tipo("Despacho", &tipos), Ok(()));
        assert_eq!(
            conferir_tipo("Parecer", &tipos),
            Err(ErroCriar::TipoAusente)
        );
        assert_eq!(
            conferir_tipo("Despacho", &["Despacho".into(), "Despacho".into()]),
            Err(ErroCriar::TipoAmbiguo)
        );
    }
    #[test]
    fn lista_fora_da_forma_falha_fechado() {
        assert_eq!(
            tipos_oferecidos("<html></html>"),
            Err(ErroCriar::ListaForaDaForma)
        );
    }
    #[test]
    fn novo_unico_com_numero_confere() {
        // Caso de Todos src/todos/documentos.py::_conferir_criacao @323eb01
        let antes = [doc("1", Some("000001"), "Ofício 000001")];
        let depois = [
            antes[0].clone(),
            doc("2", Some("000002"), "Despacho 000002"),
        ];
        assert_eq!(
            conferir_criacao(&antes, &depois, "Despacho"),
            Ok("000002".into())
        );
    }
    #[test]
    fn nenhum_novo_e_distinto_de_efeito_incerto() {
        let antes = [doc("1", Some("000001"), "Ofício 000001")];
        assert_eq!(
            conferir_criacao(&antes, &antes, "Despacho"),
            Err(ErroCriar::NaoAplicado)
        );
        let depois = [
            antes[0].clone(),
            doc("2", Some("000002"), "Despacho 000002"),
            doc("3", Some("000003"), "Despacho 000003"),
        ];
        assert_eq!(
            conferir_criacao(&antes, &depois, "Despacho"),
            Err(ErroCriar::EfeitoIncerto)
        );
        let sem_numero = [antes[0].clone(), doc("2", None, "Despacho")];
        assert_eq!(
            conferir_criacao(&antes, &sem_numero, "Despacho"),
            Err(ErroCriar::EfeitoIncerto)
        );
    }
    #[test]
    fn tipo_errado_e_documento_antigo_alterado_sao_incertos() {
        let antes = [doc("1", Some("000001"), "Ofício 000001")];
        let tipo = [antes[0].clone(), doc("2", Some("000002"), "Parecer 000002")];
        assert_eq!(
            conferir_criacao(&antes, &tipo, "Despacho"),
            Err(ErroCriar::EfeitoIncerto)
        );
        let antigo = [
            doc("1", Some("000001"), "Ofício editado 000001"),
            doc("2", Some("000002"), "Despacho 000002"),
        ];
        assert_eq!(
            conferir_criacao(&antes, &antigo, "Despacho"),
            Err(ErroCriar::EfeitoIncerto)
        );
    }

    #[test]
    fn link_direto_confere_acao_processo_e_origem_sem_executar_js() {
        // Caso de Todos src/todos/documentos.py::_tipos_da_tela @323eb01
        let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
        let href = format!(
            "controlador.php?acao=documento_gerar&id_procedimento=10&infra_hash={}",
            "a".repeat(64)
        );
        let html = format!(
            "<table id=\"tblSeries\"><tr><td><a href=\"{href}\">Despacho</a></td></tr></table>"
        );
        assert_eq!(
            link_direto_do_tipo(&instalacao, &html, "10", "Despacho"),
            Ok(format!("https://sei.exemplo.gov.br/sei/{href}"))
        );
        assert_eq!(
            link_direto_do_tipo(&instalacao, &html, "11", "Despacho"),
            Err(ErroCriar::EnderecoDoTipoNaoReconhecido)
        );
        let script = r#"<table id="tblSeries"><tr><td>
            <a href="javascript:void(0);" onclick="escolher(3);">Despacho</a>
            </td></tr></table>"#;
        assert_eq!(
            link_direto_do_tipo(&instalacao, script, "10", "Despacho"),
            Err(ErroCriar::LinkDoTipoNaoNavegavel)
        );
    }

    #[test]
    fn cadastro_so_aceita_tipo_e_controles_publicados_habilitados() {
        // Caso de Todos tests/paginas_reais/documento_gerar.html @323eb01
        let html = r#"<form id="frmDocumentoCadastro" method="post">
            <label id="lblSerieTitulo">Despacho</label>
            <input type="text" id="txtDescricao" name="txtDescricao">
            <input type="radio" id="optPublico" name="rdoNivelAcesso" value="0">
            <input type="radio" id="optRestrito" name="rdoNivelAcesso" value="1" disabled="disabled">
            </form>"#;
        assert_eq!(
            conferir_cadastro(html, "Despacho", Some("Ensaio"), Some("publico")),
            Ok(())
        );
        assert_eq!(
            conferir_cadastro(html, "Ofício", None, None),
            Err(ErroCriar::TipoDoCadastroDiferente)
        );
        assert_eq!(
            conferir_cadastro(html, "Despacho", None, Some("restrito")),
            Err(ErroCriar::NivelNaoOferecido)
        );
        assert_eq!(
            conferir_cadastro(html, "Despacho", None, Some("sigiloso")),
            Err(ErroCriar::NivelNaoOferecido)
        );
        let sem_campo = html.replace("id=\"txtDescricao\"", "id=\"outro\"");
        assert_eq!(
            conferir_cadastro(&sem_campo, "Despacho", Some("texto"), None),
            Err(ErroCriar::DescricaoNaoOferecida)
        );
        assert_eq!(
            conferir_cadastro("<html></html>", "Despacho", None, None),
            Err(ErroCriar::CadastroForaDaForma)
        );
    }

}
