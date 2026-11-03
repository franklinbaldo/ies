//! RED/GREEN do JSON publicado: só o observado, sem URL, hash ou dica.
use super::*;
use crate::arvore_processo::{ArvoreLida, DocumentoNaArvore, ProcessoNaArvore};
use crate::processo_obter::ResolvidoPor;

fn aberto(unidade: Option<&str>) -> ProcessoAberto {
    ProcessoAberto {
        arvore: ArvoreLida {
            processo: ProcessoNaArvore {
                id: "10".into(),
                nup: "00001.000001/2026-01".into(),
                tipo: "Tipo Sintetico".into(),
            },
            documentos: vec![
                DocumentoNaArvore {
                    id: "21".into(),
                    rotulo: "Despacho 0000101".into(),
                    numero_sei: Some("0000101".into()),
                },
                DocumentoNaArvore {
                    id: "22".into(),
                    rotulo: "E-mail".into(),
                    numero_sei: None,
                },
            ],
        },
        unidade: unidade.map(str::to_owned),
        resolvido_por: ResolvidoPor::Caixa,
        requisicoes: 3,
    }
}

#[test]
fn forma_do_json() {
    let nup = NumeroProcesso::novo("00001.000001/2026-01").unwrap();
    let valor = json_do_processo(&nup, &aberto(Some("UNIDADE TESTE")), false);
    assert_eq!(
        valor,
        json!({
            "numero": "00001.000001/2026-01",
            "id_procedimento": "10",
            "tipo": "Tipo Sintetico",
            "unidade": "UNIDADE TESTE",
            "resolvido_por": "caixa",
            "requisicoes": 3,
            "documentos": [
                {"id_documento": "21", "rotulo": "Despacho 0000101", "numero_sei": "0000101"},
                {"id_documento": "22", "rotulo": "E-mail", "numero_sei": null}
            ],
            "nao_lidos": ["andamentos", "marcadores", "propriedades"]
        })
    );
}

#[test]
fn sem_documentos_tira_a_chave_e_unidade_nao_observada_e_null() {
    let nup = NumeroProcesso::novo("00001.000001/2026-01").unwrap();
    let valor = json_do_processo(&nup, &aberto(None), true);
    assert!(valor.get("documentos").is_none());
    assert!(valor["unidade"].is_null());
    assert_eq!(valor["numero"], "00001.000001/2026-01");
    let texto = valor.to_string();
    assert!(!texto.contains("infra_hash") && !texto.contains("controlador.php"));
}

#[test]
fn todos_os_caminhos_tem_nome_estavel() {
    let nomes: Vec<_> = [
        ResolvidoPor::IndiceLink,
        ResolvidoPor::IndiceEntrada,
        ResolvidoPor::PesquisaRapida,
        ResolvidoPor::Caixa,
    ]
    .iter()
    .map(|p| p.como_texto())
    .collect();
    assert_eq!(nomes, ["indice_link", "indice_entrada", "pesquisa_rapida", "caixa"]);
}
