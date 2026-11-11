//! #332 — plano do anexo externo derivado da tela, sem executar JavaScript.
//! O recibo temporário não comprova o cadastro do documento.

use super::recibo::ReciboUpload;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum FalhaAnexo {
    /// O tokenizador de atributos recusou a tag.
    Atributo(#[source] crate::html_atributos::ErroAtributoHtml),
    /// O tokenizador HTML recusou a tela inteira.
    HtmlNaoAnalisavel(#[source] tl::ParseError),
    /// `new infraUpload(...)` sem a URL literal como segundo argumento.
    UrlDoUploadIlegivel,
    /// A URL de upload não é `controlador.php?acao=documento_upload_anexo&...`.
    UrlDoUploadForaDoControlador,
    /// A tela não publica exatamente um `new infraUpload(...)`.
    UploadDaTelaNaoEhUnico,
    LinhaInvalida,
    ReciboIncompativel,
}

#[derive(Clone, Copy)]
enum Campo {
    NomeUpload,
    Nome,
    Tipo,
    Tamanho,
    DataHora,
}

enum Coluna {
    Campo(Campo),
    TamanhoFormatado,
    Literal(String),
}

/// O destino assinado fica apenas na memória, nunca na saída ou em Debug.
pub(crate) struct PlanoAnexo {
    upload: String,
    colunas: Vec<Coluna>,
}

impl PlanoAnexo {
    pub(crate) fn url_upload(&self) -> &str {
        &self.upload
    }

    /// O campo hdnAnexos tem a ordem de objTabelaAnexos.adicionar na tela.
    pub(crate) fn linha_do_anexo(&self, recibo: &ReciboUpload) -> Result<String, FalhaAnexo> {
        let mut valores = Vec::with_capacity(self.colunas.len());
        for coluna in &self.colunas {
            let valor = match coluna {
                Coluna::Campo(Campo::NomeUpload) => recibo.nome_upload.clone(),
                Coluna::Campo(Campo::Nome) => recibo.nome.clone(),
                Coluna::Campo(Campo::Tipo) => recibo.tipo.clone(),
                Coluna::Campo(Campo::Tamanho) => recibo.tamanho_bytes.to_string(),
                Coluna::Campo(Campo::DataHora) => recibo.data_hora.clone(),
                Coluna::TamanhoFormatado => formatar_tamanho(recibo.tamanho_bytes),
                Coluna::Literal(texto) => texto.clone(),
            };
            // infraTabelaDinamica separa colunas por ±. Não permitir que
            // o recibo ou um literal da tela crie colunas adicionais.
            if valor.contains('±') || valor.contains(['\r', '\n']) {
                return Err(FalhaAnexo::ReciboIncompativel);
            }
            valores.push(valor);
        }
        Ok(valores.join("±"))
    }
}

/// A função observada infraFormatarTamanhoBytes usa Math.round de duas casas.
fn formatar_tamanho(bytes: u64) -> String {
    let (base, sufixo): (u128, &str) = if bytes > (1 << 40) {
        (1 << 40, "Tb")
    } else if bytes > (1 << 30) {
        (1 << 30, "Gb")
    } else if bytes > (1 << 20) {
        (1 << 20, "Mb")
    } else {
        (1024, "Kb")
    };
    let centesimos = (u128::from(bytes) * 100 + base / 2) / base;
    let inteiro = centesimos / 100;
    let resto = centesimos % 100;
    if resto == 0 {
        format!("{inteiro} {sufixo}")
    } else if resto % 10 == 0 {
        format!("{inteiro}.{} {sufixo}", resto / 10)
    } else {
        format!("{inteiro}.{resto:02} {sufixo}")
    }
}

/// O segundo argumento de `new infraUpload(...)`: a URL relativa do upload.
/// Só o destino é conferido; a forma da chamada não muda o que é enviado.
fn ler_url_upload(fonte: &str) -> Result<String, FalhaAnexo> {
    let (argumentos, _) = fonte
        .split_once(')')
        .ok_or(FalhaAnexo::UrlDoUploadIlegivel)?;
    let url = argumentos
        .split(',')
        .nth(1)
        .map(str::trim)
        .and_then(|texto| texto.strip_prefix('\''))
        .and_then(|texto| texto.strip_suffix('\''))
        .ok_or(FalhaAnexo::UrlDoUploadIlegivel)?;
    // Somente a URL relativa que a própria tela publica. O fluxo Web
    // ainda confere origem, assinatura, sessão e identidade.
    if !url.starts_with("controlador.php?acao=documento_upload_anexo&")
        || url.contains(['\'', '"', '\r', '\n'])
    {
        return Err(FalhaAnexo::UrlDoUploadForaDoControlador);
    }
    Ok(url.to_owned())
}

fn separar_colunas(fonte: &str) -> Result<Vec<&str>, FalhaAnexo> {
    let mut colunas = Vec::new();
    let mut inicio = 0_usize;
    let mut aspas = None;
    let mut escape = false;
    let mut parenteses = 0_u8;
    for (indice, caractere) in fonte.char_indices() {
        if let Some(delimitador) = aspas {
            if escape {
                escape = false;
            } else if caractere == '\\' {
                escape = true;
            } else if caractere == delimitador {
                aspas = None;
            }
            continue;
        }
        match caractere {
            '\'' | '"' => aspas = Some(caractere),
            '(' => parenteses = parenteses.checked_add(1).ok_or(FalhaAnexo::LinhaInvalida)?,
            ')' => parenteses = parenteses.checked_sub(1).ok_or(FalhaAnexo::LinhaInvalida)?,
            ',' if parenteses == 0 => {
                colunas.push(
                    fonte
                        .get(inicio..indice)
                        .ok_or(FalhaAnexo::LinhaInvalida)?
                        .trim(),
                );
                inicio = indice + 1;
            }
            _ => {}
        }
    }
    if aspas.is_some() || escape || parenteses != 0 {
        return Err(FalhaAnexo::LinhaInvalida);
    }
    colunas.push(fonte.get(inicio..).ok_or(FalhaAnexo::LinhaInvalida)?.trim());
    if colunas.is_empty() || colunas.iter().any(|coluna| coluna.is_empty()) {
        return Err(FalhaAnexo::LinhaInvalida);
    }
    Ok(colunas)
}

fn ler_colunas(fonte: &str) -> Result<Vec<Coluna>, FalhaAnexo> {
    let mut colunas = Vec::new();
    for token in separar_colunas(fonte)? {
        let coluna = match token {
            "arr['nome_upload']" => Coluna::Campo(Campo::NomeUpload),
            "arr['nome']" => Coluna::Campo(Campo::Nome),
            "arr['tipo']" => Coluna::Campo(Campo::Tipo),
            "arr['tamanho']" => Coluna::Campo(Campo::Tamanho),
            "arr['data_hora']" => Coluna::Campo(Campo::DataHora),
            "infraFormatarTamanhoBytes(arr['tamanho'])" => Coluna::TamanhoFormatado,
            _ => {
                let texto = token
                    .strip_prefix('\'')
                    .and_then(|texto| texto.strip_suffix('\''))
                    .ok_or(FalhaAnexo::LinhaInvalida)?;
                // A captura UnB contém literais simples. Escapes JS ou
                // expressões não observados não são interpretados.
                if texto.contains(['\'', '\\']) {
                    return Err(FalhaAnexo::LinhaInvalida);
                }
                Coluna::Literal(texto.to_owned())
            }
        };
        colunas.push(coluna);
    }
    Ok(colunas)
}

fn corpo_do_script(html: &str, inicio: usize) -> Option<&str> {
    let abertura = html.get(inicio..)?;
    let corpo = abertura.find('>')? + 1;
    let fim = corpo + abertura.get(corpo..)?.find("</script>")?;
    abertura.get(corpo..fim)
}

/// Interpreta apenas declarações literais em script inline ativo da tela.
/// Não executa JS, não inventa URL e não aceita colunas de template inerte.
pub(crate) fn ler_plano(html: &str) -> Result<PlanoAnexo, FalhaAnexo> {
    let dom =
        tl::parse(html, tl::ParserOptions::default()).map_err(FalhaAnexo::HtmlNaoAnalisavel)?;
    let tags = |seletor: &str| -> Vec<&tl::HTMLTag<'_>> {
        dom.query_selector(seletor)
            .into_iter()
            .flatten()
            .filter_map(|no| no.get(dom.parser()).and_then(tl::Node::as_tag))
            .collect()
    };
    let inertes: Vec<_> = tags("template")
        .into_iter()
        .map(|tag| tag.boundaries(dom.parser()))
        .collect();
    let mut uploads = Vec::new();
    let mut linhas = Vec::new();
    for tag in tags("script") {
        let inicio = tag.boundaries(dom.parser()).0;
        if inertes
            .iter()
            .any(|&(primeiro, ultimo)| (primeiro..=ultimo).contains(&inicio))
        {
            continue;
        }
        let tipo =
            crate::html_atributos::valor_do_atributo(tag, "type").map_err(FalhaAnexo::Atributo)?;
        if tipo
            .as_deref()
            .is_some_and(|tipo| !tipo.eq_ignore_ascii_case("text/javascript"))
        {
            continue;
        }
        if crate::html_atributos::atributo_presente(tag, "src").map_err(FalhaAnexo::Atributo)? {
            continue;
        }
        let Some(fonte) = corpo_do_script(html, inicio) else {
            continue;
        };
        for chamada in fonte.split("new infraUpload(").skip(1) {
            uploads.push(ler_url_upload(chamada)?);
        }
        for chamada in fonte.split("objTabelaAnexos.adicionar([").skip(1) {
            let (colunas, _) = chamada.split_once("]);").ok_or(FalhaAnexo::LinhaInvalida)?;
            linhas.push(ler_colunas(colunas)?);
        }
    }
    let [upload] = uploads.as_slice() else {
        return Err(FalhaAnexo::UploadDaTelaNaoEhUnico);
    };
    let [colunas] = linhas.as_slice() else {
        return Err(FalhaAnexo::LinhaInvalida);
    };
    Ok(PlanoAnexo {
        upload: upload.clone(),
        colunas: colunas
            .iter()
            .map(|coluna| match coluna {
                Coluna::Campo(campo) => Coluna::Campo(*campo),
                Coluna::TamanhoFormatado => Coluna::TamanhoFormatado,
                Coluna::Literal(texto) => Coluna::Literal(texto.clone()),
            })
            .collect(),
    })
}

#[cfg(test)]
mod testes {
    use super::*;

    const COLUNAS: &str = "arr['nome_upload'],arr['nome'],arr['data_hora'],arr['tamanho'],infraFormatarTamanhoBytes(arr['tamanho']),'operador','Unidade2'";

    fn tela(terceiro: &str, colunas: &str) -> String {
        format!(
            r#"<html><body><form id="frmDocumentoCadastro"><input type="hidden" name="hdnAnexos" /></form>
<script type="text/javascript">
funcaoConclusao = function(arr) {{
  objTabelaAnexos.adicionar([{colunas}]);
}};
objUpload = new infraUpload('frmAnexos','controlador.php?acao=documento_upload_anexo&infra_hash=SINTETICO'{terceiro});
objTabelaAnexos = new infraTabelaDinamica('tblAnexos','hdnAnexos',false,false);
</script></body></html>"#
        )
    }

    fn recibo() -> ReciboUpload {
        ReciboUpload {
            nome_upload: "temp-42".to_owned(),
            nome: "prova.pdf".to_owned(),
            tipo: "application/pdf".to_owned(),
            tamanho_bytes: 68,
            data_hora: "11/10/2026 11:07:50".to_owned(),
        }
    }

    #[test]
    fn dado_cadastro_unb_4_quando_recebo_upload_entao_gera_linha_da_tabela() {
        // Dado a ordem da funcaoConclusao capturada no Todos/UnB,
        // quando monto hdnAnexos, então preservo todas as colunas observadas.
        let plano = ler_plano(&tela("", COLUNAS)).expect("cadastro UnB 4");
        assert!(plano.url_upload().contains("acao=documento_upload_anexo"));
        assert_eq!(
            plano.linha_do_anexo(&recibo()),
            Ok("temp-42±prova.pdf±11/10/2026 11:07:50±68±0.07 Kb±operador±Unidade2".to_owned())
        );
    }

    #[test]
    fn dado_sei_5_quando_terceiro_argumento_booleano_entao_mesmo_plano() {
        // PEN3 SEI 5.0.4 publica new infraUpload(..., true).
        for terceiro in [", true", ", false"] {
            let plano = ler_plano(&tela(terceiro, COLUNAS)).expect("cadastro SEI 5");
            assert_eq!(
                plano.linha_do_anexo(&recibo()).unwrap().split('±').count(),
                7
            );
        }
    }

    #[test]
    fn dado_colunas_reordenadas_quando_recibo_entao_respeita_a_tela() {
        let colunas = "'Unidade2',arr['nome'],arr['nome_upload'],infraFormatarTamanhoBytes(arr['tamanho']),arr['data_hora']";
        let plano = ler_plano(&tela("", colunas)).expect("ordem da tela");
        assert_eq!(
            plano.linha_do_anexo(&recibo()),
            Ok("Unidade2±prova.pdf±temp-42±0.07 Kb±11/10/2026 11:07:50".to_owned())
        );
    }

    #[test]
    fn dado_upload_ambiguo_ou_ausente_quando_leio_entao_recuso() {
        for (indice, (tela, esperado)) in [
            (
                tela("", COLUNAS).replace("new infraUpload", "outraCoisa"),
                FalhaAnexo::UploadDaTelaNaoEhUnico,
            ),
            (
                format!("{}{}", tela("", COLUNAS), tela("", COLUNAS)),
                FalhaAnexo::UploadDaTelaNaoEhUnico,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(
                ler_plano(&tela).map(|_| ()),
                Err(esperado),
                "indice {indice}"
            );
        }
    }

    #[test]
    fn dado_coluna_fora_da_forma_ou_ambigua_quando_leio_entao_recuso() {
        for colunas in [
            "arr['campo_desconhecido']",
            "",
            "arr['nome'],funcao()",
            "arr['nome'",
        ] {
            assert!(matches!(
                ler_plano(&tela("", colunas)),
                Err(FalhaAnexo::LinhaInvalida)
            ));
        }
    }

    #[test]
    fn dado_recibo_com_separador_quando_monto_linha_entao_recuso() {
        let plano = ler_plano(&tela("", COLUNAS)).expect("cadastro");
        let mut resposta = recibo();
        resposta.nome = "prova±outro.pdf".to_owned();
        assert_eq!(
            plano.linha_do_anexo(&resposta),
            Err(FalhaAnexo::ReciboIncompativel)
        );
    }

    #[test]
    fn dado_only_template_quando_leio_entao_nao_invento_upload() {
        let falso = format!("<template>{}</template>", tela("", COLUNAS));
        assert_eq!(
            ler_plano(&falso).map(|_| ()),
            Err(FalhaAnexo::UploadDaTelaNaoEhUnico)
        );
    }
}
