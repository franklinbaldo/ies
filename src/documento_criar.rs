//! Núcleo offline da criação de documento interno (#246). Nenhuma rede.
use crate::arvore_processo::DocumentoNaArvore;
use crate::html_atributos::{nos_com_id, valor_do_atributo};
use crate::formulario_web::decodificar_atributo;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroCriar {
    ListaForaDaForma,
    TipoAusente,
    TipoAmbiguo,
    NaoAplicado,
    EfeitoIncerto,
}
impl ErroCriar {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::ListaForaDaForma => "lista_de_tipos_fora_da_forma_observada",
            Self::TipoAusente => "tipo_de_documento_nao_oferecido",
            Self::TipoAmbiguo => "tipo_de_documento_ambiguo",
            Self::NaoAplicado => "documento_criado_nao_comprovado",
            Self::EfeitoIncerto => "efeito_incerto_na_criacao_de_documento",
        }
    }
}
/// Apenas nomes publicados em table#tblSeries; nunca interpreta JS como código.
pub(crate) fn tipos_oferecidos(html: &str) -> Result<Vec<String>, ErroCriar> {
    let err = ErroCriar::ListaForaDaForma;
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| err)?;
    let tabelas = nos_com_id(&dom, "tblSeries").map_err(|_| err)?;
    let [tabela] = tabelas.as_slice() else { return Err(err) };
    let parser = dom.parser();
    let tabela = tabela.get(parser).and_then(tl::Node::as_tag).ok_or(err)?;
    if !tabela.name().as_utf8_str().eq_ignore_ascii_case("table") {
        return Err(err);
    }
    let mut tipos = Vec::new();
    for tag in tabela.children().all(parser).iter().filter_map(tl::Node::as_tag) {
        if !tag.name().as_utf8_str().eq_ignore_ascii_case("a") { continue; }
        let href = valor_do_atributo(tag, "href").map_err(|_| err)?;
        if href.is_none() { continue; }
        let onclick = valor_do_atributo(tag, "onclick").map_err(|_| err)?;
        if matches!(onclick.as_deref().map(str::trim), Some("escolher(-1);" | "escolher(-1)")) {
            continue;
        }
        let nome = tag.inner_text(parser).split_whitespace().collect::<Vec<_>>().join(" ");
        let nome = decodificar_atributo(&nome).ok_or(err)?;
        if !nome.is_empty() { tipos.push(nome); }
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
pub(crate) fn conferir_criacao(
    antes: &[DocumentoNaArvore], depois: &[DocumentoNaArvore], tipo: &str,
) -> Result<String, ErroCriar> {
    use ErroCriar as E;
    let mut ids = std::collections::HashSet::new();
    for doc in depois {
        if !ids.insert(doc.id.as_str()) { return Err(E::EfeitoIncerto); }
    }
    for original in antes {
        if !depois.iter().any(|doc| doc == original) {
            return Err(E::EfeitoIncerto);
        }
    }
    let novos: Vec<_> = depois.iter().filter(|doc| !antes.iter().any(|d| d.id == doc.id)).collect();
    let novo = match novos.as_slice() {
        [] => return Err(E::NaoAplicado),
        [doc] => *doc,
        _ => return Err(E::EfeitoIncerto),
    };
    let numero = novo.numero_sei.as_deref()
        .filter(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
        .ok_or(E::EfeitoIncerto)?;
    if novo.rotulo != format!("{tipo} {numero}")
        || antes.iter().any(|d| d.numero_sei.as_deref() == Some(numero))
        || depois.iter().filter(|d| d.numero_sei.as_deref() == Some(numero)).count() != 1
    {
        return Err(E::EfeitoIncerto);
    }
    Ok(numero.to_owned())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::arvore_processo::NaturezaDoDocumento;
    fn doc(id: &str, numero: Option<&str>, rotulo: &str) -> DocumentoNaArvore {
        DocumentoNaArvore {
            id: id.into(), rotulo: rotulo.into(), numero_sei: numero.map(str::to_owned),
            pasta: None,
            natureza: NaturezaDoDocumento {
                icone: None, cancelado: None, motivo_do_cancelamento: None, dica: None,
            },
        }
    }
    #[test]
    fn lista_tipo_ausente_ambiguo_e_externo() {
        // Caso de Todos src/todos/documentos.py::_tipos_da_tela @323eb01
        let html = r##"<table id="tblSeries"><tr><td>
            <a href="controlador.php?acao=documento_gerar">Despacho</a>
            <a href="#" onclick="escolher(-1);">Externo</a>
            <a href="#" onclick="escolher(3);">Ofício</a>
            </td></tr></table>"##;
        let tipos = tipos_oferecidos(html).unwrap();
        assert_eq!(tipos, ["Despacho", "Ofício"]);
        assert_eq!(conferir_tipo("Despacho", &tipos), Ok(()));
        assert_eq!(conferir_tipo("Parecer", &tipos), Err(ErroCriar::TipoAusente));
        assert_eq!(conferir_tipo("Despacho", &["Despacho".into(), "Despacho".into()]), Err(ErroCriar::TipoAmbiguo));
    }
    #[test]
    fn lista_fora_da_forma_falha_fechado() {
        assert_eq!(tipos_oferecidos("<html></html>"), Err(ErroCriar::ListaForaDaForma));
    }
    #[test]
    fn novo_unico_com_numero_confere() {
        // Caso de Todos src/todos/documentos.py::_conferir_criacao @323eb01
        let antes = [doc("1", Some("000001"), "Ofício 000001")];
        let depois = [antes[0].clone(), doc("2", Some("000002"), "Despacho 000002")];
        assert_eq!(conferir_criacao(&antes, &depois, "Despacho"), Ok("000002".into()));
    }
    #[test]
    fn nenhum_novo_e_distinto_de_efeito_incerto() {
        let antes = [doc("1", Some("000001"), "Ofício 000001")];
        assert_eq!(conferir_criacao(&antes, &antes, "Despacho"), Err(ErroCriar::NaoAplicado));
        let depois = [antes[0].clone(), doc("2", Some("000002"), "Despacho 000002"), doc("3", Some("000003"), "Despacho 000003")];
        assert_eq!(conferir_criacao(&antes, &depois, "Despacho"), Err(ErroCriar::EfeitoIncerto));
        let sem_numero = [antes[0].clone(), doc("2", None, "Despacho")];
        assert_eq!(conferir_criacao(&antes, &sem_numero, "Despacho"), Err(ErroCriar::EfeitoIncerto));
    }
    #[test]
    fn tipo_errado_e_documento_antigo_alterado_sao_incertos() {
        let antes = [doc("1", Some("000001"), "Ofício 000001")];
        let tipo = [antes[0].clone(), doc("2", Some("000002"), "Parecer 000002")];
        assert_eq!(conferir_criacao(&antes, &tipo, "Despacho"), Err(ErroCriar::EfeitoIncerto));
        let antigo = [doc("1", Some("000001"), "Ofício editado 000001"), doc("2", Some("000002"), "Despacho 000002")];
        assert_eq!(conferir_criacao(&antes, &antigo, "Despacho"), Err(ErroCriar::EfeitoIncerto));
    }
}
