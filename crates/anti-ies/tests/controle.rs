use anti_ies::{Estado, despachar_com_controle};
use iesei::instalacao::Instalacao;
use iesei::observacao::{
    Cobertura, FormaDoControle, ObservacaoDoControle, ProcessoNaLista, Proveniencia,
    TabelaObservada, UnidadeExibida,
};

fn sinteticamente_observado() -> ObservacaoDoControle<ProcessoNaLista> {
    ObservacaoDoControle {
        instalacao: Instalacao::interpretar("https://sei.exemplo/sei").unwrap(),
        unidade: UnidadeExibida::nova("PPI").unwrap(),
        forma: FormaDoControle::Resumida {
            recebidos: Cobertura::Observada(TabelaObservada::nova(Some(0), None, vec![]).unwrap()),
            gerados: Cobertura::Observada(TabelaObservada::nova(Some(0), None, vec![]).unwrap()),
        },
        proveniencia: Proveniencia::Cassette {
            cartao: "sintetica".to_owned(),
        },
    }
}

#[test]
fn controle_resumido_usa_a_unidade_observada_e_nao_finge_conteudo() {
    let observado = sinteticamente_observado();
    let resposta = despachar_com_controle(
        "GET",
        "/sei/controlador.php?acao=procedimento_controlar",
        Some(&observado),
    );
    assert_eq!(resposta.estado, Estado::Ok);
    assert_eq!(resposta.tipo_conteudo, "text/html; charset=utf-8");
    assert!(resposta.corpo.contains("<a id='lnkInfraUnidade'>PPI</a>"));
    assert!(resposta.corpo.contains("tblProcessosRecebidos"));
    assert!(resposta.corpo.contains("tblProcessosGerados"));
}

#[test]
fn sem_dados_observados_nao_gera_lista_vazia() {
    let resposta = despachar_com_controle(
        "GET",
        "/sei/controlador.php?acao=procedimento_controlar",
        None,
    );
    assert_eq!(resposta.estado, Estado::DadosInsuficientes);
    assert!(!resposta.corpo.contains("tblProcessosRecebidos"));
}

#[test]
fn html_da_unidade_e_escapado_antes_de_servir() {
    let mut observado = sinteticamente_observado();
    observado.unidade = UnidadeExibida::nova("A<&B").unwrap();
    let resposta = despachar_com_controle(
        "GET",
        "/sei/controlador.php?acao=procedimento_controlar",
        Some(&observado),
    );
    assert!(resposta.corpo.contains("A&lt;&amp;B"));
    assert!(!resposta.corpo.contains("A<&B"));
}

#[test]
fn acoes_nao_implementadas_continuam_recusadas() {
    let observado = sinteticamente_observado();
    let resposta = despachar_com_controle(
        "GET",
        "/sei/controlador.php?acao=documento_visualizar",
        Some(&observado),
    );
    assert_eq!(resposta.estado, Estado::NaoImplementado);
}
