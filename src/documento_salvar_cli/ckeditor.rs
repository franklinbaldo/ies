//! Gravação pelo JSON que o próprio editor do SEI 5 publica.
use super::ErroEditorNativo;
use super::copia_renderizada::Secao;
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::documento_versoes::VersaoDocumento;
use crate::instalacao::Instalacao;
use crate::tela_html::{HtmlServido, arvore};
use crate::transporte::{cliente_https, ler_limitado, recusar_status};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
enum Versao {
    Numero(u32),
    Texto(String),
}

impl Versao {
    fn lida(&self) -> Option<VersaoDocumento> {
        match self {
            Self::Numero(n) => VersaoDocumento::nova(*n),
            Self::Texto(n) => VersaoDocumento::interpretar(n),
        }
        .ok()
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Sei {
    url_salvar: String,
    versao: Versao,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Atributos {
    principal: bool,
    somente_leitura: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Configuracao {
    sei: Sei,
    #[serde(deserialize_with = "secoes_em_ordem")]
    initial_data: Vec<Secao>,
    roots_attributes: BTreeMap<String, Atributos>,
}

fn secoes_em_ordem<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<Secao>, D::Error> {
    struct Leitor;
    impl<'de> serde::de::Visitor<'de> for Leitor {
        type Value = Vec<Secao>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("seções publicadas na ordem do editor")
        }
        fn visit_map<M: serde::de::MapAccess<'de>>(
            self,
            mut mapa: M,
        ) -> Result<Self::Value, M::Error> {
            let mut secoes: Vec<Secao> = Vec::new();
            while let Some((nome, html)) = mapa.next_entry()? {
                if secoes.iter().any(|s| s.nome == nome) {
                    return Err(serde::de::Error::custom("seção repetida"));
                }
                secoes.push(Secao { nome, html });
            }
            Ok(secoes)
        }
    }
    deserializer.deserialize_map(Leitor)
}

impl Configuracao {
    pub(super) fn ler(html: &HtmlServido) -> Result<Option<Self>, ErroEditorNativo> {
        let dom = arvore(html);
        let configs: Vec<_> = dom
            .elementos("script")
            .filter(|s| {
                !s.presente("src")
                    && s.atributo("type")
                        .is_none_or(|t| t.eq_ignore_ascii_case("text/javascript"))
            })
            .filter_map(|s| {
                let texto = s.texto();
                let ini = texto
                    .trim_start()
                    .strip_prefix("window.INFRA_EDITOR_CONFIG")?;
                Some(ini.trim_start().strip_prefix('=').and_then(|valor| {
                    let mut leitor = serde_json::Deserializer::from_str(valor).into_iter::<Self>();
                    leitor.next()?.ok()
                }))
            })
            .collect();
        match configs.as_slice() {
            [] => Ok(None),
            [_] => configs
                .into_iter()
                .next()
                .flatten()
                .map(Some)
                .ok_or(ErroEditorNativo::EditorForaDaForma),
            _ => Err(ErroEditorNativo::EditorForaDaForma),
        }
    }

    pub(super) fn secoes(&self) -> &[Secao] {
        &self.initial_data
    }

    pub(super) fn principal(&self) -> Option<&str> {
        let mut achados = self.secoes().iter().filter(|secao| {
            self.roots_attributes
                .get(&secao.nome)
                .is_some_and(|a| a.principal && !a.somente_leitura)
        });
        let nome = &achados.next()?.nome;
        achados.next().is_none().then_some(nome)
    }

    pub(super) fn versao(&self) -> Option<VersaoDocumento> {
        self.sei.versao.lida()
    }

    fn url(&self, instalacao: &Instalacao, documento: &str) -> Option<String> {
        let relativo = html_escape::decode_html_entities(&self.sei.url_salvar);
        let consulta = relativo.strip_prefix("controlador_rest.php?")?;
        if relativo.contains(['#', '\\', '\'', '"'])
            || !relativo.bytes().all(|b| (33..=126).contains(&b))
        {
            return None;
        }
        let consulta = ConsultaDoControlador::ler(consulta).ok()?;
        if consulta.unico("id_documento") != Ok(documento)
            || consulta.unico("acao_rest") != Ok("editor_salvar_conteudo")
            || consulta.valores("acao").next().is_some()
            || consulta.valores("acao_ajax").next().is_some()
            || !consulta
                .unico("infra_hash")
                .is_ok_and(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit()))
        {
            return None;
        }
        Some(format!("{}/{relativo}", instalacao.url_base()))
    }

    fn corpo(&self, principal: &str, editado: &str) -> Result<Vec<u8>, ErroEditorNativo> {
        #[derive(Serialize)]
        struct SecaoEnviada<'a> {
            nome: &'a str,
            html: &'a str,
        }
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Pedido<'a> {
            secoes_conteudo: Vec<SecaoEnviada<'a>>,
            ignorar_nova_versao: &'static str,
            versao: &'a Versao,
        }
        let htmls = super::htmls_para_post(self.secoes(), principal, editado);
        let secoes: Vec<_> = self
            .secoes()
            .iter()
            .zip(&htmls)
            .map(|(s, html)| SecaoEnviada {
                nome: &s.nome,
                html,
            })
            .collect();
        serde_json::to_vec(&Pedido {
            secoes_conteudo: secoes,
            ignorar_nova_versao: "N",
            versao: &self.sei.versao,
        })
        .map_err(ErroEditorNativo::JsonDoEditor)
    }

    pub(super) fn salvar(
        &self,
        instalacao: &Instalacao,
        alvo: &crate::processo_obter::DocumentoNaArvoreAberto,
        referer: &str,
        principal: &str,
        editado: &str,
    ) -> Result<Option<VersaoDocumento>, ErroEditorNativo> {
        #[derive(Deserialize)]
        struct Aceite {
            versao: Versao,
        }
        let url = self
            .url(instalacao, &alvo.id_documento)
            .ok_or(ErroEditorNativo::FormularioDeOutroDocumento)?;
        let corpo = self.corpo(principal, editado)?;
        let envio = cliente_https::Envio {
            tipo: "application/json",
            origem: format!("https://{}", instalacao.host()),
            referer,
            corpo: &corpo,
        };
        let resposta = cliente_https::resposta_web(cliente_https::requisitar(
            "POST",
            &url,
            &alvo.cookie,
            Some(&envio),
        )?);
        if resposta.status_code != 200 {
            return Err(ErroEditorNativo::Tela(recusar_status(resposta)));
        }
        let recebido = ler_limitado(resposta.corpo, crate::MAX_HTML_BYTES)?;
        let json: Aceite =
            serde_json::from_slice(&recebido).map_err(ErroEditorNativo::JsonDoEditor)?;
        Ok(json.versao.lida())
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn conserva_ordem_e_html_no_json() {
        let html = HtmlServido::de_teste(
            r#"<script>window.INFRA_EDITOR_CONFIG = {"sei":{"versao":1,"urlSalvar":"controlador_rest.php"},"initialData":{"txaEditor_40":"<p>Fixo\\r\\r\\nOutro\\r</p>","txaEditor_3":"<p>Anterior</p>"},"rootsAttributes":{"txaEditor_40":{"principal":false,"somenteLeitura":true},"txaEditor_3":{"principal":true,"somenteLeitura":false}}};</script>"#,
        );
        let config = Configuracao::ler(&html).unwrap().unwrap();
        let editado = crate::documento_salvar_cli::html_para_envio("<p title=\"x\r\r\né 🔎");
        let corpo = config.corpo(config.principal().unwrap(), &editado).unwrap();
        let json: serde_json::Value = serde_json::from_slice(&corpo).unwrap();
        assert_eq!(json["secoesConteudo"][0]["html"], "<p>Fixo\\nOutro\\n</p>");
        assert_eq!(config.secoes()[0].html, "<p>Fixo\\r\\r\\nOutro\\r</p>");
        assert!(!String::from_utf8(corpo).unwrap().contains("\\\\r"));
        assert_eq!(json["secoesConteudo"][1]["html"], editado);
        assert_eq!(editado, "<p title=\"x\né 🔎");
        assert_eq!(json["ignorarNovaVersao"], "N");
        assert_eq!(json["versao"], 1);
    }
}
