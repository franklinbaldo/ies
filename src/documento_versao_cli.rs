//! Versão e foto na saída de `ies documento DOC` (Refs #88, #207;
//! contracts/cli/documento-obter-versoes.md): o pedido a partir de
//! `--versao` e `--saida`; os campos `versao`, `versao_atual` e
//! `arquivo`; e a gravação idempotente de `DOC.vN.html`, conferida para o
//! lote inteiro antes da primeira gravação.
use crate::copia_documento::ErroCopia;
use crate::documento_versoes::VersaoDocumento;
use crate::foto_documento::{self, FotoGravada};
use crate::processo_obter::{DocumentoObtido, PedidoDeVersoes};
use serde_json::{Map, Value, json};
use std::path::Path;

/// O pedido de versões; `None` para `--versao 0` (validado antes, saída 1).
pub(crate) fn pedido(versao: Option<u32>, saida: bool) -> Option<PedidoDeVersoes> {
    let versao = match versao {
        Some(n) => Some(VersaoDocumento::nova(n).ok()?),
        None => None,
    };
    Some(PedidoDeVersoes {
        versao,
        foto: saida,
    })
}

/// As fotos do lote, na ordem (`None` para o externo): todas conferidas
/// antes da primeira gravação, então uma divergência não grava nenhuma.
pub(crate) fn gravar_fotos(
    diretorio: &Path,
    obtidos: &[DocumentoObtido],
) -> Result<Vec<Option<FotoGravada>>, ErroCopia> {
    for d in obtidos {
        if let Some(f) = &d.versoes.foto {
            foto_documento::planejar(diretorio, &d.numero, f.versao, &f.html)?;
        }
    }
    obtidos
        .iter()
        .map(|d| {
            d.versoes
                .foto
                .as_ref()
                .map(|f| foto_documento::gravar(diretorio, &d.numero, f.versao, &f.html))
                .transpose()
        })
        .collect()
}

/// O objeto `arquivo` de um documento gravado em `--saida`.
pub(crate) fn json_do_arquivo(nome: &str, bytes: u64, sha256: &str, gravado: bool) -> Value {
    json!({
        "nome": nome,
        "bytes": bytes,
        "sha256": sha256,
        "gravado": gravado,
        "motivo": if gravado { Value::Null } else { json!("igual") },
    })
}

/// `versao`, `versao_atual` e `versao_atual_motivo`.
pub(crate) fn inserir_campos(objeto: &mut Map<String, Value>, d: &DocumentoObtido) {
    let v = &d.versoes;
    objeto.insert("versao".into(), json!(v.lida.map(VersaoDocumento::numero)));
    let (atual, motivo) = match v.atual {
        Ok(n) => (json!(n.numero()), Value::Null),
        Err(sem) => (Value::Null, json!(sem.motivo())),
    };
    objeto.insert("versao_atual".into(), atual);
    objeto.insert("versao_atual_motivo".into(), motivo);
}

/// Linha humana sobre a versão, quando a atual é conhecida.
pub(crate) fn linha_humana(d: &DocumentoObtido) -> Option<String> {
    let v = &d.versoes;
    let atual = v.atual.ok()?;
    let lida = v
        .lida
        .map_or_else(String::new, |n| format!("versão {} lida; ", n.numero()));
    Some(format!("Versões: {lida}atual {}.", atual.numero()))
}

/// Linha humana do arquivo gravado (ou já igual) em `--saida`.
pub(crate) fn linha_do_arquivo(nome: &str, bytes: u64, gravado: bool) -> String {
    if gravado {
        format!("Arquivo gravado em {nome} ({bytes} bytes).")
    } else {
        format!("Arquivo {nome} já existe com o mesmo SHA-256 ({bytes} bytes); nada gravado.")
    }
}

#[cfg(test)]
pub(crate) mod testes {
    use super::*;
    use crate::arvore_processo::{NaturezaDoDocumento, ProcessoNaArvore};
    use crate::processo_obter::{FotoDaVersao, SemVersaoAtual, VersoesDoDocumento};

    pub(crate) fn documento(versoes: VersoesDoDocumento) -> DocumentoObtido {
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
            unidade: Some("TESTE".into()),
            resolvido_por: crate::processo_obter::ResolvidoPor::PesquisaRapida,
            texto: "a".into(),
            anexo: None,
            requisicoes: 6,
            versoes,
        }
    }

    pub(crate) fn v(n: u32) -> VersaoDocumento {
        VersaoDocumento::nova(n).unwrap()
    }

    pub(crate) fn diretorio() -> std::path::PathBuf {
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

    const HTML: &str =
        "<html><head><title>SEI/X - 0011519 - Despacho</title></head><body><p>a</p></body></html>";

    pub(crate) fn com_foto(numero: &str, n: u32, html: &str) -> DocumentoObtido {
        let mut d = documento(VersoesDoDocumento {
            lida: Some(v(n)),
            atual: Ok(v(n)),
            foto: Some(FotoDaVersao {
                versao: v(n),
                html: html.into(),
            }),
        });
        d.numero = numero.into();
        d
    }

    fn nomes(pasta: &Path) -> Vec<String> {
        let mut nomes: Vec<String> = std::fs::read_dir(pasta)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        nomes.sort();
        nomes
    }

    #[test]
    fn pedido_de_versoes_pelas_opcoes() {
        assert_eq!(pedido(None, false), Some(PedidoDeVersoes::default()));
        assert_eq!(
            pedido(None, true),
            Some(PedidoDeVersoes {
                versao: None,
                foto: true
            })
        );
        assert_eq!(
            pedido(Some(3), false),
            Some(PedidoDeVersoes {
                versao: Some(v(3)),
                foto: false
            })
        );
        assert_eq!(pedido(Some(0), true), None);
    }

    #[test]
    fn lote_grava_so_as_fotos_e_repetir_e_no_op() {
        let pasta = diretorio();
        let mut externo = documento(VersoesDoDocumento {
            lida: None,
            atual: Err(SemVersaoAtual::NaoServidaNoConteudo),
            foto: None,
        });
        externo.numero = "0011521".into();
        let lote = [
            com_foto("0011519", 2, HTML),
            externo,
            com_foto("0011520", 1, HTML),
        ];
        let gravadas = gravar_fotos(&pasta, &lote).unwrap();
        assert_eq!(gravadas.len(), 3);
        assert!(gravadas[1].is_none());
        let primeira = gravadas[0].as_ref().unwrap();
        assert_eq!(
            (primeira.nome.as_str(), primeira.gravado),
            ("0011519.v2.html", true)
        );
        assert_eq!(nomes(&pasta), ["0011519.v2.html", "0011520.v1.html"]);
        let de_novo = gravar_fotos(&pasta, &lote).unwrap();
        assert!(de_novo.iter().flatten().all(|f| !f.gravado));
        assert_eq!(nomes(&pasta), ["0011519.v2.html", "0011520.v1.html"]);
        std::fs::remove_dir_all(pasta).unwrap();
    }

    #[test]
    fn divergencia_no_lote_nao_grava_nenhuma() {
        let pasta = diretorio();
        gravar_fotos(&pasta, &[com_foto("0011520", 1, HTML)]).unwrap();
        let lote = [
            com_foto("0011519", 2, HTML),
            com_foto("0011520", 1, "<html>outra</html>"),
        ];
        assert_eq!(gravar_fotos(&pasta, &lote), Err(ErroCopia::FotoDivergente));
        assert_eq!(nomes(&pasta), ["0011520.v1.html"]);
        std::fs::remove_dir_all(pasta).unwrap();
    }

    #[test]
    fn campos_de_versao_e_do_arquivo() {
        let d = com_foto("0011519", 2, HTML);
        let mut objeto = Map::new();
        inserir_campos(&mut objeto, &d);
        assert_eq!(
            Value::Object(objeto),
            json!({"versao": 2, "versao_atual": 2, "versao_atual_motivo": null})
        );
        assert_eq!(
            linha_humana(&d).as_deref(),
            Some("Versões: versão 2 lida; atual 2.")
        );
        assert_eq!(
            json_do_arquivo("0011519.v2.html", 7, "ab", false),
            json!({"nome": "0011519.v2.html", "bytes": 7, "sha256": "ab", "gravado": false, "motivo": "igual"})
        );
        let nao = documento(VersoesDoDocumento {
            lida: None,
            atual: Err(SemVersaoAtual::NaoServidaNoConteudo),
            foto: None,
        });
        let mut objeto = Map::new();
        inserir_campos(&mut objeto, &nao);
        assert_eq!(
            objeto["versao_atual_motivo"],
            json!("nao_servida_no_conteudo")
        );
        assert_eq!(linha_humana(&nao), None);
    }
}
