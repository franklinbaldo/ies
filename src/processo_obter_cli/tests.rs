//! RED/GREEN do JSON publicado: só o observado, sem URL, hash ou dica.
use super::*;
use crate::arvore_processo::{ArvoreLida, DocumentoNaArvore, PastaNaArvore, ProcessoNaArvore};
use crate::historico_processo::{AndamentosLidos, TipoDoHistorico};
use crate::processo::Andamento;
use crate::processo_obter::{Andamentos, ResolvidoPor};

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
                    pasta: None,
                },
                DocumentoNaArvore {
                    id: "22".into(),
                    rotulo: "E-mail".into(),
                    numero_sei: None,
                    pasta: None,
                },
            ],
            pastas: Vec::new(),
            pastas_nao_carregadas: false,
        },
        unidade: unidade.map(str::to_owned),
        resolvido_por: ResolvidoPor::Caixa,
        andamentos: Andamentos::Omitidos,
        requisicoes: 3,
    }
}

fn com_andamentos(total: u64) -> ProcessoAberto {
    let mut aberto = aberto(Some("UNIDADE TESTE"));
    aberto.requisicoes = 4;
    aberto.andamentos = Andamentos::Lidos(AndamentosLidos {
        tipo: TipoDoHistorico::Resumido,
        total_declarado: total,
        itens: vec![
            Andamento {
                ocorrido_em: Some("01/01/2026 10:00".into()),
                unidade: Some("U1".into()),
                usuario: Some("p1".into()),
                descricao: "Processo sintetico gerado".into(),
            },
            Andamento {
                ocorrido_em: Some("02/01/2026 10:00".into()),
                unidade: Some("U2".into()),
                usuario: None,
                descricao: "Andamento sintetico".into(),
            },
        ],
    });
    aberto
}

#[test]
fn andamentos_lidos_saem_com_total_faixa_e_itens_e_deixam_nao_lidos() {
    let nup = NumeroProcesso::novo("00001.000001/2026-01").unwrap();
    let valor = json_do_processo(&nup, &com_andamentos(2), true);
    assert_eq!(
        valor["andamentos"],
        json!({
            "tipo": "resumido",
            "total_declarado": 2,
            "faixa_observada": {"inicio": 1, "fim": 2},
            "itens": [
                {"ocorrido_em": "01/01/2026 10:00", "unidade": "U1", "usuario": "p1",
                 "descricao": "Processo sintetico gerado"},
                {"ocorrido_em": "02/01/2026 10:00", "unidade": "U2", "usuario": null,
                 "descricao": "Andamento sintetico"}
            ]
        })
    );
    assert_eq!(valor["nao_lidos"], json!(["marcadores", "propriedades"]));
    assert!(valor.get("andamentos_nao_lidos_motivo").is_none());
    assert_eq!(valor["requisicoes"], 4);
}

#[test]
fn link_nao_publicado_sai_null_com_motivo_e_sem_andamentos_tira_a_chave() {
    let nup = NumeroProcesso::novo("00001.000001/2026-01").unwrap();
    let mut sem_link = aberto(None);
    sem_link.andamentos = Andamentos::LinkNaoPublicadoNaArvore;
    let valor = json_do_processo(&nup, &sem_link, true);
    assert!(valor["andamentos"].is_null());
    assert!(valor.get("andamentos").is_some());
    assert_eq!(
        valor["andamentos_nao_lidos_motivo"],
        "link_do_historico_nao_publicado_na_arvore"
    );
    assert_eq!(
        valor["nao_lidos"],
        json!(["andamentos", "marcadores", "propriedades"])
    );
    let omitidos = json_do_processo(&nup, &aberto(None), true);
    assert!(omitidos.get("andamentos").is_none());
    assert_eq!(
        omitidos["andamentos_nao_lidos_motivo"],
        "omitidos_por_sem_andamentos"
    );
}

#[test]
fn menos_linhas_que_o_total_nao_alegam_completude() {
    let nup = NumeroProcesso::novo("00001.000001/2026-01").unwrap();
    let valor = json_do_processo(&nup, &com_andamentos(49), true);
    assert_eq!(valor["andamentos"]["total_declarado"], 49);
    assert_eq!(
        valor["andamentos"]["faixa_observada"],
        json!({"inicio": 1, "fim": 2})
    );
    let texto = humano(&nup, &com_andamentos(49), true);
    assert!(texto.contains("2 de 49"), "{texto}");
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
                {"id_documento": "21", "rotulo": "Despacho 0000101", "numero_sei": "0000101", "pasta": null},
                {"id_documento": "22", "rotulo": "E-mail", "numero_sei": null, "pasta": null}
            ],
            "pastas": [],
            "nao_lidos": ["andamentos", "marcadores", "propriedades"],
            "andamentos_nao_lidos_motivo": "omitidos_por_sem_andamentos"
        })
    );
}

#[test]
fn sem_documentos_tira_a_chave_e_unidade_nao_observada_e_null() {
    let nup = NumeroProcesso::novo("00001.000001/2026-01").unwrap();
    let valor = json_do_processo(&nup, &aberto(None), true);
    assert!(valor.get("documentos").is_none());
    assert!(valor.get("pastas").is_none());
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
    assert_eq!(
        nomes,
        ["indice_link", "indice_entrada", "pesquisa_rapida", "caixa"]
    );
}

#[test]
fn documentos_dizem_a_pasta_e_as_pastas_saem_com_rotulo_e_total() {
    let nup = NumeroProcesso::novo("00001.000001/2026-01").unwrap();
    let mut em_pastas = aberto(None);
    for d in &mut em_pastas.arvore.documentos {
        d.pasta = Some("PASTA1".into());
    }
    em_pastas.arvore.pastas = vec![PastaNaArvore {
        id: "PASTA1".into(),
        rotulo: "I".into(),
        total_declarado: 2,
    }];
    let valor = json_do_processo(&nup, &em_pastas, false);
    assert_eq!(valor["documentos"][1]["pasta"], "PASTA1");
    assert_eq!(
        valor["pastas"],
        json!([{"id": "PASTA1", "rotulo": "I", "total": 2}])
    );
    let texto = humano(&nup, &em_pastas, false);
    assert!(texto.contains("[pasta I]"), "{texto}");
    assert!(texto.contains("1 pasta(s)"), "{texto}");
}
