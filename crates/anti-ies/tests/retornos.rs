use anti_ies::{Estado, retorno_programado};
use iesei::banco_cassettes::iniciar;
use iesei::instalacao::Instalacao;
use iesei::retornos_cassettes::{Mes, Retorno, Situacao, guardar};
use rusqlite::Connection;

fn instalacao() -> Instalacao {
    Instalacao::interpretar("https://sei.exemplo/sei").unwrap()
}

#[test]
fn opera_mensal_sem_inventar_conteudo_ou_consultar_rede() {
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let origem = instalacao();
    let endereco = "/sei/controlador.php?acao=retorno_programado_listar&data_inicial=01&data_final=31";
    assert_eq!(
        retorno_programado("GET", endereco, &banco, &origem, "cartao").estado,
        Estado::DadosInsuficientes
    );
    let mes = Mes {
        instalacao: origem.clone(),
        data_inicial: "01".into(),
        data_final: "31".into(),
        itens: vec![Retorno {
            situacao: Situacao::Aguardando,
            numero: "NUP-A".into(),
            unidade: "U1".into(),
            data_programada: "15/10/2026".into(),
            retornado_em: None,
            prazo_restante: None,
        }],
    };
    guardar(&mut banco, "cartao", &mes).unwrap();
    let resposta = retorno_programado("GET", endereco, &banco, &origem, "cartao");
    assert_eq!(resposta.estado, Estado::Ok);
    assert_eq!(resposta.tipo_conteudo, "text/html; charset=utf-8");
    assert!(resposta.corpo.contains("NUP-A"));
    assert!(resposta.corpo.contains("Aguardando Retorno de Outras Unidades"));
    assert!(!resposta.corpo.contains("infra_hash"));
    assert_eq!(
        retorno_programado("POST", endereco, &banco, &origem, "cartao").estado,
        Estado::MetodoNaoPermitido
    );
    assert_eq!(
        retorno_programado("GET", "/sei/controlador.php?acao=retorno_programado_listar&data_inicial=02&data_final=31", &banco, &origem, "cartao").estado,
        Estado::DadosInsuficientes
    );
    assert_eq!(
        retorno_programado("GET", "/sei/controlador.php?acao=retorno_programado_listar&data_inicial=01", &banco, &origem, "cartao").estado,
        Estado::RequisicaoInvalida
    );
}
