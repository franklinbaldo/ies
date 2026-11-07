use iesei::observacao::{
    Cobertura, FaixaObservada, FormaDoControle, ObservacaoDoControle, Proveniencia,
    TabelaObservada, UnidadeExibida,
};

#[test]
fn distingue_lista_nao_observada_de_lista_vazia_observada() {
    let desconhecida: Cobertura<TabelaObservada<()>> = Cobertura::NaoObservada;
    let vazia = Cobertura::Observada(TabelaObservada::nova(Some(0), None, vec![]).unwrap());
    assert_ne!(desconhecida, vazia);
}

#[test]
fn nao_inventa_controle_resumido_quando_falta_uma_tabela() {
    let instalacao = iesei::instalacao::Instalacao::interpretar("https://sei.exemplo/sei").unwrap();
    let observado = ObservacaoDoControle {
        instalacao,
        unidade: UnidadeExibida::nova("PPI").unwrap(),
        forma: FormaDoControle::Resumida {
            recebidos: Cobertura::Observada(
                TabelaObservada::<()>::nova(Some(0), None, vec![]).unwrap(),
            ),
            gerados: Cobertura::NaoObservada,
        },
        proveniencia: Proveniencia::Cassette {
            cartao: "fixture-sintetica".into(),
        },
    };
    assert!(!observado.suficiente_para_servir());
}

#[test]
fn controle_com_duas_tabelas_observadas_pode_ser_servido() {
    let instalacao = iesei::instalacao::Instalacao::interpretar("https://sei.exemplo/sei").unwrap();
    let observado = ObservacaoDoControle {
        instalacao,
        unidade: UnidadeExibida::nova("PPI").unwrap(),
        forma: FormaDoControle::Resumida {
            recebidos: Cobertura::Observada(
                TabelaObservada::<()>::nova(Some(0), None, vec![]).unwrap(),
            ),
            gerados: Cobertura::Observada(
                TabelaObservada::<()>::nova(Some(0), None, vec![]).unwrap(),
            ),
        },
        proveniencia: Proveniencia::Cassette {
            cartao: "fixture-sintetica".into(),
        },
    };
    assert!(observado.suficiente_para_servir());
}

#[test]
fn faixa_invalida_e_linhas_incompativeis_sao_recusadas() {
    assert!(FaixaObservada::nova(0, 1).is_none());
    assert!(FaixaObservada::nova(5, 4).is_none());
    let faixa = FaixaObservada::nova(1, 3).unwrap();
    assert!(TabelaObservada::nova(Some(2), Some(faixa), vec![1, 2, 3]).is_err());
    assert!(UnidadeExibida::nova("  ").is_none());
}
