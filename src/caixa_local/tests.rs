use super::*;
use serde_json::json;

fn caixa() -> String {
    json!([
        {
            "unidade": "PPI", "origem": "web", "categoria": "recebidos",
            "total_declarado": 1, "faixa_observada": {"inicio": 1, "fim": 1},
            "processos": [{"id":"0042","numero":"000123/2026","visualizado":false}]
        },
        {
            "unidade": "PPI", "origem": "web", "categoria": "gerados",
            "total_declarado": 0, "faixa_observada": null, "processos": []
        }
    ])
    .to_string()
}
#[test]
fn encontra_nup_exato_de_arquivo_de_caixa_sem_rede() {
    let observada = interpretar(&caixa(), Some("ppi")).unwrap();
    assert_eq!(observada.unidade.as_deref(), Some("PPI"));
    assert_eq!(observada.caixa().recebidos.processos[0].numero, "000123/2026");
    assert_eq!(observada.caixa().recebidos.processos[0].id.as_str(), "0042");
}
#[test]
fn sigiloso_exportado_e_preservado_e_ausencia_antiga_e_nao_sigiloso() {
    let antiga = interpretar(&caixa(), None).unwrap();
    assert!(!antiga.caixa().recebidos.processos[0].sigiloso);
    let nova = caixa().replace(
        "\"visualizado\":false",
        "\"visualizado\":false,\"sigiloso\":true",
    );
    let observada = interpretar(&nova, None).unwrap();
    assert!(observada.caixa().recebidos.processos[0].sigiloso);
}
#[test]
fn recusa_total_que_nao_comprova_todas_as_linhas_da_categoria() {
    let json = caixa().replace("\"total_declarado\":1", "\"total_declarado\":2");
    assert_eq!(
        interpretar(&json, None).unwrap_err(),
        ErroCaixaLocal::TotalNaoComprovado {
            categoria: Some(Categoria::Recebidos)
        }
    );
}
#[test]
fn recusa_unidade_trocada_sem_abrir_sessao() {
    assert_eq!(
        interpretar(&caixa(), Some("OUTRA")).unwrap_err(),
        ErroCaixaLocal::UnidadeDiferenteDaSolicitada
    );
}
#[test]
fn recusa_identificador_repetido_sem_selecionar_a_primeira_linha() {
    let mut objeto: serde_json::Value = serde_json::from_str(&caixa()).unwrap();
    objeto[0]["total_declarado"] = json!(2);
    let repetido = objeto[0]["processos"][0].clone();
    objeto[0]["processos"]
        .as_array_mut()
        .unwrap()
        .push(repetido);
    assert_eq!(
        interpretar(&objeto.to_string(), None).unwrap_err(),
        ErroCaixaLocal::IdentificadorRepetido {
            categoria: Some(Categoria::Recebidos)
        }
    );
}
#[test]
fn aceita_exportacao_de_caixa_multipagina_gerada_pelo_ies() {
    let mut exportada: serde_json::Value = serde_json::from_str(&caixa()).unwrap();
    exportada[0]["origem"] = json!("web_com_paginas_privadas");
    exportada[1]["origem"] = json!("web_com_paginas_privadas");
    let observada = interpretar(&exportada.to_string(), Some("PPI")).unwrap();
    assert_eq!(observada.caixa().recebidos.processos[0].numero, "000123/2026");
    assert_eq!(observada.caixa().recebidos.processos[0].id.as_str(), "0042");
}

#[test]
fn exportacao_multipagina_continua_exigindo_totais_comprovados() {
    let mut exportada: serde_json::Value = serde_json::from_str(&caixa()).unwrap();
    exportada[0]["origem"] = json!("web_com_paginas_privadas");
    exportada[1]["origem"] = json!("web_com_paginas_privadas");
    exportada[0]["total_declarado"] = json!(2);
    assert_eq!(
        interpretar(&exportada.to_string(), None).unwrap_err(),
        ErroCaixaLocal::TotalNaoComprovado {
            categoria: Some(Categoria::Recebidos)
        }
    );
}

#[test]
fn apenas_as_duas_origens_de_exportacao_reais_sao_aceitas() {
    for origem in ["WEB", "web_replay", "web_com_paginas_privadas_fake"] {
        let mut exportada: serde_json::Value = serde_json::from_str(&caixa()).unwrap();
        exportada[0]["origem"] = json!(origem);
        exportada[1]["origem"] = json!(origem);
        assert_eq!(
            interpretar(&exportada.to_string(), None).unwrap_err(),
            ErroCaixaLocal::OrigemNaoWeb
        );
    }
}

#[test]
fn recusa_origem_de_cassette_ou_json_nao_proveniente_de_exportacao() {
    let json = caixa().replace("\"origem\":\"web\"", "\"origem\":\"replay\"");
    assert!(!json.contains("\"origem\":\"web\""));
    assert_eq!(
        interpretar(&json, None).unwrap_err(),
        ErroCaixaLocal::OrigemNaoWeb
    );
}

#[test]
fn recusa_listas_fora_da_forma_recebidos_gerados() {
    let base: serde_json::Value = serde_json::from_str(&caixa()).unwrap();
    let mut trocada = base.clone();
    trocada.as_array_mut().unwrap().reverse();
    let mut uma = base.clone();
    uma.as_array_mut().unwrap().pop();
    let mut detalhada = base.clone();
    detalhada[1]["categoria"] = json!("detalhada");
    for valor in [trocada, uma, detalhada] {
        assert_eq!(
            interpretar(&valor.to_string(), None).unwrap_err(),
            ErroCaixaLocal::ListasExportadasNaoSaoRecebidosEGerados
        );
    }
    let mut unidades = base.clone();
    unidades[1]["unidade"] = json!("OUTRA");
    assert_eq!(
        interpretar(&unidades.to_string(), None).unwrap_err(),
        ErroCaixaLocal::UnidadeDasListasExportadasDiferente
    );
    let mut origens = base.clone();
    origens[1]["origem"] = json!("web_com_paginas_privadas");
    assert_eq!(
        interpretar(&origens.to_string(), None).unwrap_err(),
        ErroCaixaLocal::OrigemDasListasExportadasDiferente
    );
}

#[test]
fn exportacao_no_formato_anterior_e_recusada() {
    let antiga = json!({
        "unidade": "PPI", "origem": "web",
        "recebidos": {"total_declarado": 0, "processos": []},
        "gerados": {"total_declarado": 0, "processos": []}
    });
    assert_eq!(
        interpretar(&antiga.to_string(), None).unwrap_err(),
        ErroCaixaLocal::FormatoInvalido
    );
}

fn detalhada() -> serde_json::Value {
    json!([{
        "unidade": "PPI", "origem": "web", "categoria": null,
        "total_declarado": 2, "faixa_observada": {"inicio": 1, "fim": 2},
        "processos": [
            {"id":"0042","numero":"000123/2026","visualizado":null,"sigiloso":null},
            {"id":"43","numero":"000124/2026","visualizado":true,"sigiloso":false}
        ]
    }])
}

#[test]
fn aceita_lista_unica_sem_categoria_da_visao_detalhada() {
    let observada = interpretar(&detalhada().to_string(), Some("ppi")).unwrap();
    assert_eq!(observada.unidade.as_deref(), Some("PPI"));
    let ListasLocais::SemCategoria(processos) = observada.listas else {
        panic!("esperava a lista sem categoria");
    };
    assert_eq!(processos.len(), 2);
    assert_eq!(processos[0].id.as_str(), "0042");
    assert_eq!(processos[0].numero, "000123/2026");
    assert_eq!(processos[0].visualizado, None);
    assert_eq!(processos[0].sigiloso, None);
    assert_eq!(processos[1].visualizado, Some(true));
    assert_eq!(processos[1].sigiloso, Some(false));
}

#[test]
fn recusa_lista_unica_detalhada_da_pr_146_como_formato_invalido() {
    let mut antiga = detalhada();
    antiga[0]["categoria"] = json!("detalhada");
    assert_eq!(
        interpretar(&antiga.to_string(), None).unwrap_err(),
        ErroCaixaLocal::FormatoInvalido
    );
}

#[test]
fn lista_unica_com_categoria_ou_par_sem_categoria_nao_e_aceito() {
    let mut recebidos = detalhada();
    recebidos[0]["categoria"] = json!("recebidos");
    let mut par: serde_json::Value = serde_json::from_str(&caixa()).unwrap();
    par[0]["categoria"] = json!(null);
    par[1]["categoria"] = json!(null);
    for valor in [recebidos, par] {
        assert_eq!(
            interpretar(&valor.to_string(), None).unwrap_err(),
            ErroCaixaLocal::ListasExportadasNaoSaoRecebidosEGerados
        );
    }
}

#[test]
fn lista_sem_categoria_exige_total_e_identidades_comprovados() {
    let mut total = detalhada();
    total[0]["total_declarado"] = json!(3);
    let mut id = detalhada();
    id[0]["processos"][1]["id"] = json!("x");
    let mut repetido = detalhada();
    repetido[0]["processos"][1]["id"] = json!("0042");
    let mut numero = detalhada();
    numero[0]["processos"][1]["numero"] = json!(" 000124/2026");
    let casos = [
        (
            total,
            ErroCaixaLocal::TotalNaoComprovado { categoria: None },
        ),
        (
            id,
            ErroCaixaLocal::IdentificadorInvalido { categoria: None },
        ),
        (
            repetido,
            ErroCaixaLocal::IdentificadorRepetido { categoria: None },
        ),
        (numero, ErroCaixaLocal::NumeroInvalido { categoria: None }),
    ];
    for (valor, esperado) in casos {
        assert_eq!(interpretar(&valor.to_string(), None).unwrap_err(), esperado);
    }
}

#[test]
fn lista_sem_categoria_confere_origem_e_unidade() {
    let mut origem = detalhada();
    origem[0]["origem"] = json!("replay");
    assert_eq!(
        interpretar(&origem.to_string(), None).unwrap_err(),
        ErroCaixaLocal::OrigemNaoWeb
    );
    assert_eq!(
        interpretar(&detalhada().to_string(), Some("OUTRA")).unwrap_err(),
        ErroCaixaLocal::UnidadeDiferenteDaSolicitada
    );
}

#[test]
fn subcodigos_da_lista_sem_categoria_sao_estaveis() {
    let casos = [
        (
            ErroCaixaLocal::TotalNaoComprovado { categoria: None },
            "total_da_lista_sem_categoria_do_arquivo_nao_comprovado",
        ),
        (
            ErroCaixaLocal::IdentificadorInvalido { categoria: None },
            "id_da_lista_sem_categoria_do_arquivo_invalido",
        ),
        (
            ErroCaixaLocal::NumeroInvalido { categoria: None },
            "numero_da_lista_sem_categoria_do_arquivo_invalido",
        ),
        (
            ErroCaixaLocal::IdentificadorRepetido { categoria: None },
            "id_da_lista_sem_categoria_do_arquivo_repetido",
        ),
    ];
    for (erro, subcodigo) in casos {
        assert_eq!(erro.subcodigo(), subcodigo);
    }
}

impl CaixaObservadaLocal {
    /// As listas `[recebidos, gerados]`; outra forma é falha do teste.
    pub(crate) fn caixa(&self) -> &CaixaDeEntrada {
        match &self.listas {
            ListasLocais::Categorias(caixa) => caixa,
            ListasLocais::SemCategoria(_) => panic!("esperava recebidos e gerados"),
        }
    }
}
