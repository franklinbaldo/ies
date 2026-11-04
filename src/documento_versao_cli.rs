//! Versão e cópia de trabalho na saída de `ies documento DOC` (Refs #88;
//! contracts/cli/documento-obter-versoes.md): o pedido a partir de
//! `--versao`, `--saida` e `--sem-copia`; os campos `versao`,
//! `versao_atual`, `copia` e motivos; e a gravação de `DOC.vN.html` (com a
//! base e o manifesto de `documento salvar`), 0600, sem sobrescrever.
use crate::copia_documento::{self, ErroCopia, FonteHtml, IdentidadeCopia};
use crate::documento::NumeroDocumento;
use crate::documento_versoes::VersaoDocumento;
use crate::processo_obter::{DocumentoObtido, PedidoDeVersoes};
use serde_json::{Map, Value, json};
use std::path::Path;

/// A cópia gravada: nome relativo a `--saida` e bytes da cópia.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CopiaGravada {
    pub(crate) arquivo: String,
    pub(crate) bytes: usize,
}

/// O pedido de versões; `None` para `--versao 0` (validado antes, saída 1).
pub(crate) fn pedido(versao: Option<u32>, saida: bool, sem_copia: bool) -> Option<PedidoDeVersoes> {
    let versao = match versao {
        Some(n) => Some(VersaoDocumento::nova(n).ok()?),
        None => None,
    };
    Some(PedidoDeVersoes {
        versao,
        copia: saida && !sem_copia,
    })
}

/// Grava a cópia quando a leitura a preparou; nada quando não.
pub(crate) fn gravar_copia(
    diretorio: &Path,
    origem: &str,
    d: &DocumentoObtido,
) -> Result<Option<CopiaGravada>, ErroCopia> {
    let Ok(copia) = &d.versoes.copia else {
        return Ok(None);
    };
    let documento = NumeroDocumento::novo(&d.numero).map_err(|_| ErroCopia::IdentidadeInvalida)?;
    let identidade = IdentidadeCopia {
        documento,
        id_interno: d.id_documento.clone(),
        instalacao: origem.to_owned(),
        versao: copia.versao,
    };
    let fonte = FonteHtml::LeituraAtualNaoAssinada { html: &copia.html };
    copia_documento::preparar(diretorio, &identidade, fonte)?;
    Ok(Some(CopiaGravada {
        arquivo: format!("{}.v{}.html", d.numero, copia.versao.numero()),
        bytes: copia.html.len(),
    }))
}

/// `versao`, `versao_atual`, `versao_atual_motivo`, `copia` e `copia_motivo`.
pub(crate) fn inserir_campos(
    objeto: &mut Map<String, Value>,
    d: &DocumentoObtido,
    gravada: Option<&CopiaGravada>,
) {
    let v = &d.versoes;
    objeto.insert("versao".into(), json!(v.lida.map(VersaoDocumento::numero)));
    let (atual, motivo) = match v.atual {
        Ok(n) => (json!(n.numero()), Value::Null),
        Err(sem) => (Value::Null, json!(sem.motivo())),
    };
    objeto.insert("versao_atual".into(), atual);
    objeto.insert("versao_atual_motivo".into(), motivo);
    let (copia, motivo) = match (&v.copia, gravada) {
        (Ok(_), Some(g)) => (json!({"arquivo": g.arquivo, "bytes": g.bytes}), Value::Null),
        // Preparada, mas sem `--saida` não há onde gravar.
        (Ok(_), None) => (Value::Null, json!("nao_pedida")),
        (Err(sem), _) => (Value::Null, json!(sem.motivo())),
    };
    objeto.insert("copia".into(), copia);
    objeto.insert("copia_motivo".into(), motivo);
}

/// Linha humana sobre versão e cópia, quando há algo a dizer.
pub(crate) fn linha_humana(d: &DocumentoObtido, gravada: Option<&CopiaGravada>) -> Option<String> {
    let v = &d.versoes;
    let atual = v.atual.ok()?;
    let lida = v
        .lida
        .map_or_else(String::new, |n| format!("versão {} lida; ", n.numero()));
    let copia = match (gravada, &v.copia) {
        (Some(g), _) => format!("cópia de trabalho {} ({} bytes)", g.arquivo, g.bytes),
        (None, Err(sem)) => format!("sem cópia de trabalho ({})", sem.motivo()),
        (None, Ok(_)) => "sem cópia de trabalho".to_owned(),
    };
    Some(format!("Versões: {lida}atual {}; {copia}.", atual.numero()))
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::arvore_processo::{NaturezaDoDocumento, ProcessoNaArvore};
    use crate::processo_obter::{CopiaDeTrabalho, SemCopia, SemVersaoAtual, VersoesDoDocumento};

    fn documento(versoes: VersoesDoDocumento) -> DocumentoObtido {
        DocumentoObtido {
            numero: "0011519".into(),
            id_documento: "42".into(),
            rotulo: "Despacho 0011519".into(),
            tipo: "Despacho".into(),
            natureza: NaturezaDoDocumento {
                icone: Some("documento_interno".into()),
                cancelado: Some(false),
                motivo_do_cancelamento: None,
                dica: None,
            },
            pasta: None,
            processo: ProcessoNaArvore {
                id: "10".into(),
                nup: "00001.000001/2026-01".into(),
                tipo: "Tipo".into(),
            },
            unidade: "TESTE".into(),
            texto: "a".into(),
            requisicoes: 6,
            versoes,
        }
    }

    fn v(n: u32) -> VersaoDocumento {
        VersaoDocumento::nova(n).unwrap()
    }

    fn diretorio() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "ies-versao-cli-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&path).unwrap();
        path
    }

    #[test]
    fn pedido_de_versoes_pelas_opcoes() {
        assert_eq!(pedido(None, false, false), Some(PedidoDeVersoes::default()));
        assert_eq!(pedido(None, true, true), Some(PedidoDeVersoes::default()));
        assert_eq!(
            pedido(Some(3), true, false),
            Some(PedidoDeVersoes {
                versao: Some(v(3)),
                copia: true
            })
        );
        assert_eq!(pedido(Some(0), false, false), None);
    }

    #[test]
    fn grava_copia_base_e_manifesto_e_publica_campos() {
        let pasta = diretorio();
        let html = "<html><head><title>SEI/X - 0011519 - Despacho</title></head><body><p>a</p></body></html>";
        let d = documento(VersoesDoDocumento {
            lida: None,
            atual: Ok(v(2)),
            copia: Ok(CopiaDeTrabalho {
                versao: v(2),
                html: html.into(),
            }),
        });
        let gravada = gravar_copia(&pasta, "https://sei.exemplo.gov.br/sei", &d)
            .unwrap()
            .unwrap();
        assert_eq!(gravada.arquivo, "0011519.v2.html");
        assert_eq!(
            std::fs::read_to_string(pasta.join("0011519.v2.html")).unwrap(),
            html
        );
        assert!(pasta.join(".0011519.v2.base.html").exists());
        assert!(pasta.join(".0011519.v2.meta.json").exists());
        let previa = crate::copia_documento::previa_local(
            &pasta,
            &NumeroDocumento::novo("0011519").unwrap(),
            None,
        )
        .unwrap();
        assert_eq!(previa.versao_base, 2);
        let mut objeto = Map::new();
        inserir_campos(&mut objeto, &d, Some(&gravada));
        assert_eq!(
            Value::Object(objeto),
            json!({
                "versao": null,
                "versao_atual": 2,
                "versao_atual_motivo": null,
                "copia": {"arquivo": "0011519.v2.html", "bytes": html.len()},
                "copia_motivo": null
            })
        );
        assert_eq!(
            gravar_copia(&pasta, "https://sei.exemplo.gov.br/sei", &d),
            Err(ErroCopia::ArquivoExistente)
        );
        std::fs::remove_dir_all(pasta).unwrap();
    }

    #[test]
    fn sem_copia_nada_e_gravado_e_os_motivos_saem() {
        let pasta = diretorio();
        let d = documento(VersoesDoDocumento {
            lida: Some(v(1)),
            atual: Ok(v(2)),
            copia: Err(SemCopia::VersaoHistorica),
        });
        assert_eq!(
            gravar_copia(&pasta, "https://sei.exemplo.gov.br/sei", &d),
            Ok(None)
        );
        assert_eq!(std::fs::read_dir(&pasta).unwrap().count(), 0);
        let mut objeto = Map::new();
        inserir_campos(&mut objeto, &d, None);
        assert_eq!(objeto["versao"], json!(1));
        assert_eq!(objeto["copia_motivo"], json!("versao_historica"));
        let nao = documento(VersoesDoDocumento {
            lida: None,
            atual: Err(SemVersaoAtual::NaoServidaNoConteudo),
            copia: Err(SemCopia::NaoPedida),
        });
        let mut objeto = Map::new();
        inserir_campos(&mut objeto, &nao, None);
        assert_eq!(objeto["versao_atual"], Value::Null);
        assert_eq!(
            objeto["versao_atual_motivo"],
            json!("nao_servida_no_conteudo")
        );
        assert_eq!(linha_humana(&nao, None), None);
        std::fs::remove_dir_all(pasta).unwrap();
    }
}
