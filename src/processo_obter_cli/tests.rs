//! RED/GREEN do JSON publicado: só o observado, sem URL, hash ou dica.
use super::*;
use crate::arvore_processo::{
    AbertoEm, ArvoreLida, DocumentoNaArvore, PastaNaArvore, ProcessoNaArvore, UnidadeAberta,
    VerificacaoDaPasta,
};
use crate::historico_processo::{AndamentosLidos, TipoDoHistorico};
use crate::processo::Andamento;
use crate::processo_obter::{Andamentos, Marcadores, Propriedades, ResolvidoPor};

pub(super) fn aberto(unidade: Option<&str>) -> ProcessoAberto {
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
                    natureza: crate::arvore_processo::tests::interno(),
                },
                DocumentoNaArvore {
                    id: "22".into(),
                    rotulo: "E-mail".into(),
                    numero_sei: None,
                    pasta: None,
                    natureza: crate::arvore_processo::tests::interno(),
                },
            ],
            pastas: Vec::new(),
            pastas_nao_carregadas: false,
            documentos_incluidos: true,
            aviso_leitura_parcial: None,
            anexados: Vec::new(),
            documentos_movidos: Vec::new(),
            anexado_a: None,
            aberto_em: AbertoEm::Unidades(vec![UnidadeAberta {
                sigla: "UNIDADE TESTE".into(),
                nome: "Unidade Teste".into(),
                atribuido_a: None,
            }]),
        },
        unidade: unidade.map(str::to_owned),
        resolvido_por: ResolvidoPor::Caixa,
        andamentos: Andamentos::Omitidos,
        propriedades: Propriedades::Omitidas,
        marcadores: Marcadores::Omitidos,
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
                {"ocorrido_em": "02/01/2026 10:00", "unidade": "U2",
                 "descricao": "Andamento sintetico"}
            ]
        })
    );
    assert_eq!(valor["nao_lidos"], json!(["marcadores", "propriedades"]));
    assert!(valor.get("andamentos_nao_lidos_motivo").is_none());
    assert_eq!(valor["requisicoes"], 4);
}

#[test]
fn sem_andamentos_tira_a_chave_e_so_diz_o_motivo_pedido() {
    let nup = NumeroProcesso::novo("00001.000001/2026-01").unwrap();
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
            "anexado": false,
            "anexados": [],
            "aberto_em": [{"sigla": "UNIDADE TESTE", "nome": "Unidade Teste"}],
            "documentos": [
                {"id_documento": "21", "rotulo": "Despacho 0000101", "numero_sei": "0000101",
                 "icone": "documento_interno", "cancelado": false,
                 "dica": "De: pessoa@exemplo.gov.br\nPara: outra@exemplo.gov.br"},
                {"id_documento": "22", "rotulo": "E-mail",
                 "icone": "documento_interno", "cancelado": false,
                 "dica": "De: pessoa@exemplo.gov.br\nPara: outra@exemplo.gov.br"}
            ],
            "documentos_movidos": [],
            "pastas": [],
            "nao_lidos": ["andamentos", "marcadores", "propriedades"],
            "andamentos_nao_lidos_motivo": "omitidos_por_sem_andamentos",
            "marcadores_nao_lidos_motivo": "omitidos_por_sem_marcadores",
            "propriedades_nao_lidas_motivo": "omitidas_por_sem_propriedades"
        })
    );
}

#[test]
fn sem_documentos_tira_a_chave_e_unidade_nao_observada_sai() {
    let nup = NumeroProcesso::novo("00001.000001/2026-01").unwrap();
    let valor = json_do_processo(&nup, &aberto(None), true);
    assert!(valor.get("documentos").is_none());
    assert!(valor.get("pastas").is_none());
    assert!(valor.get("unidade").is_none());
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
        verificacao: Some(VerificacaoDaPasta::PorIdentidade),
        conferidos_por_identidade: 2,
        conferidos_por_contagem: 0,
    }];
    let valor = json_do_processo(&nup, &em_pastas, false);
    assert_eq!(valor["documentos"][1]["pasta"], "PASTA1");
    assert_eq!(
        valor["pastas"],
        json!([{"id": "PASTA1", "rotulo": "I", "total": 2, "verificacao": "por_identidade",
                "conferidos_por_identidade": 2, "conferidos_por_contagem": 0}])
    );
    assert_eq!(valor["verificacao_das_pastas"], "por_identidade");
    assert!(valor.get("verificacao_das_pastas_motivo").is_none());
    let texto = humano(&nup, &em_pastas, false);
    assert!(texto.contains("[pasta I]"), "{texto}");
    assert!(texto.contains("1 pasta(s)"), "{texto}");
}

fn com_verificacoes(regras: &[VerificacaoDaPasta]) -> ProcessoAberto {
    let mut em_pastas = aberto(None);
    em_pastas.arvore.pastas = regras
        .iter()
        .enumerate()
        .map(|(i, v)| PastaNaArvore {
            id: format!("PASTA{}", i + 1),
            rotulo: "I".into(),
            total_declarado: 1,
            verificacao: Some(*v),
            conferidos_por_identidade: 1,
            conferidos_por_contagem: 0,
        })
        .collect();
    em_pastas
}

#[test]
fn a_verificacao_do_topo_e_a_mais_fraca_e_diz_o_motivo() {
    use VerificacaoDaPasta::{Mista, PorContagem, PorIdentidade};
    let nup = NumeroProcesso::novo("00001.000001/2026-01").unwrap();
    for (regras, topo) in [
        (vec![PorIdentidade, PorContagem], "por_contagem"),
        (vec![PorIdentidade, Mista], "mista"),
        (vec![Mista, PorContagem], "por_contagem"),
    ] {
        let valor = json_do_processo(&nup, &com_verificacoes(&regras), false);
        assert_eq!(valor["verificacao_das_pastas"], topo, "{regras:?}");
        assert_eq!(
            valor["verificacao_das_pastas_motivo"],
            "documentos_sem_identidade_na_barra"
        );
        assert_eq!(valor["pastas"][1]["verificacao"], regras[1].como_texto());
    }
    let todas = json_do_processo(&nup, &com_verificacoes(&[PorIdentidade]), false);
    assert_eq!(todas["verificacao_das_pastas"], "por_identidade");
    assert!(todas.get("verificacao_das_pastas_motivo").is_none());
    let texto = humano(&nup, &com_verificacoes(&[Mista]), false);
    assert!(texto.contains("mista"), "{texto}");
}

fn com_anexado() -> ProcessoAberto {
    let mut aberto = aberto(None);
    aberto.arvore.anexados = vec![ProcessoNaArvore {
        id: "20".into(),
        nup: "00002.000002/2026-02".into(),
        tipo: "Tipo Anexado".into(),
    }];
    aberto
}

#[test]
fn anexados_saem_sempre_com_numero_id_e_tipo_e_dizem_o_que_nao_foi_lido() {
    let nup = NumeroProcesso::novo("00001.000001/2026-01").unwrap();
    let esperado = json!([
        {"numero": "00002.000002/2026-02", "id_procedimento": "20", "tipo": "Tipo Anexado"}
    ]);
    for sem_documentos in [false, true] {
        let valor = json_do_processo(&nup, &com_anexado(), sem_documentos);
        assert_eq!(valor["anexados"], esperado, "{sem_documentos}");
        assert_eq!(
            valor["nao_lidos"],
            json!([
                "andamentos",
                "marcadores",
                "propriedades",
                "documentos_dos_anexados"
            ])
        );
    }
    let sem = json_do_processo(&nup, &aberto(None), true);
    assert_eq!(sem["anexados"], json!([]));
    let texto = humano(&nup, &com_anexado(), true);
    assert!(texto.contains("1 processo(s) anexado(s)"), "{texto}");
    assert!(
        texto.contains(
            "00002.000002/2026-02 (id 20, Tipo Anexado): ies processo 00002.000002/2026-02"
        ),
        "{texto}"
    );
    assert!(!humano(&nup, &aberto(None), true).contains("anexado"));
}

fn anexado_a_um_pai() -> ProcessoAberto {
    use crate::arvore_processo::tests_anexado_a::{
        ID_PAI, NUP_PAI, ORIGEM, arvore_do_anexado, href_do_pai, html_do_anexado,
    };
    let html = arvore_do_anexado(
        "10",
        "00001.000001/2026-01",
        &html_do_anexado(&href_do_pai(ID_PAI, "10"), NUP_PAI),
    );
    let instalacao = crate::instalacao::Instalacao::interpretar(ORIGEM).unwrap();
    let mut aberto = aberto(None);
    aberto.arvore.anexado_a =
        crate::arvore_processo::ler_vinculo_de_anexacao(&instalacao, &html, "10").unwrap();
    aberto
}

#[test]
fn processo_anexado_publica_o_pai_sem_url_e_com_a_dica() {
    let nup = NumeroProcesso::novo("00001.000001/2026-01").unwrap();
    for sem_documentos in [false, true] {
        let valor = json_do_processo(&nup, &anexado_a_um_pai(), sem_documentos);
        assert_eq!(valor["anexado"], true);
        assert_eq!(
            valor["anexado_a"],
            json!({"numero": "00009.000009/2026-09", "id_procedimento": "90"})
        );
        let texto = valor.to_string();
        assert!(!texto.contains("controlador.php") && !texto.contains(&"4".repeat(64)));
    }
    let nao = json_do_processo(&nup, &aberto(None), true);
    assert_eq!(nao["anexado"], false);
    assert!(nao.get("anexado_a").is_none());
    let texto = humano(&nup, &anexado_a_um_pai(), true);
    assert!(
        texto.contains(
            "Anexado ao processo 00009.000009/2026-09 (id 90): ies processo 00009.000009/2026-09"
        ),
        "{texto}"
    );
    assert!(!humano(&nup, &aberto(None), true).contains("Anexado ao"));
}
