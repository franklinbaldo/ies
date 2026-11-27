//! Projeção local de uma testemunha explicitamente escolhida do catálogo de gestão.
use crate::{Estado, Resposta, escapar};
use iesei::gestao_marcadores::{self, Estado as EstadoMarcador, Pagina};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::fmt::Write as _;

fn html(unidade: &str, pagina: &Pagina) -> String {
    let unidade = escapar(unidade);
    let mut saida = format!(
        "<html><body><a id='lnkInfraUnidade'>{unidade}</a><table summary='Tabela de Marcadores.'>"
    );
    if let Some((inicio, fim)) = pagina.faixa {
        write!(
            saida,
            "<caption>Lista de Marcadores ({} registros - {inicio} a {fim}):</caption>",
            pagina.total
        )
        .expect("String");
    } else {
        saida.push_str("<caption>Lista de Marcadores (0 registros):</caption>");
    }
    saida.push_str("<tr><th>ID</th><th>Nome</th><th>Ícone</th><th>Ações</th></tr>");
    for item in &pagina.itens {
        let acao = match item.estado {
            EstadoMarcador::Ativo => "Desativar Marcador",
            EstadoMarcador::Inativo => "Reativar Marcador",
        };
        write!(
            saida,
            "<tr><td>{}</td><td>{}</td><td><img title='{}'></td><td><img title='{acao}'></td></tr>",
            escapar(&item.id),
            escapar(&item.nome),
            escapar(&item.cor)
        )
        .expect("String");
    }
    saida.push_str("</table></body></html>");
    saida
}

fn responder_unidade(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    unidade: &str,
) -> Resposta {
    match gestao_marcadores::ler_selecionada(banco, origem, cenario, unidade) {
        Ok(Some(pagina)) => Resposta::html(html(unidade, &pagina)),
        Ok(None) | Err(_) => Resposta::nova(Estado::DadosInsuficientes),
    }
}

// O Controle detalhado tem precedência, como em crate::controlar.
fn unidade_do_controle(banco: &Connection, origem: &Instalacao, cenario: &str) -> Option<String> {
    match iesei::detalhada_cassettes::ler(banco, origem, cenario).ok()? {
        Some(controle) => {
            let configuracao = iesei::colunas_detalhe::ler(banco, origem, cenario).ok()?;
            if configuracao.is_some_and(|c| c.unidade != controle.unidade) {
                return None;
            }
            Some(controle.unidade.as_str().to_owned())
        }
        None => iesei::banco_cassettes::ler(banco, origem, cenario)
            .ok()?
            .map(|controle| controle.unidade.as_str().to_owned()),
    }
}

pub(crate) fn acrescentar_menu(
    mut resposta: Resposta,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> Resposta {
    if resposta.estado != Estado::Ok {
        return resposta;
    }
    let Some(unidade) = unidade_do_controle(banco, origem, cenario) else {
        return resposta;
    };
    if matches!(
        gestao_marcadores::ler_selecionada(banco, origem, cenario, &unidade),
        Ok(Some(_))
    ) {
        let link = format!(
            "<a href='controlador.php?acao=marcador_listar&amp;infra_hash={}'>Marcadores</a>",
            crate::protocolo_sintetico::INFRA_HASH
        );
        resposta.corpo = resposta
            .corpo
            .replacen("</body>", &format!("{link}</body>"), 1);
    }
    resposta
}

pub(crate) fn responder(
    metodo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> Resposta {
    if metodo != "GET" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let Some(unidade) = unidade_do_controle(banco, origem, cenario) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    responder_unidade(banco, origem, cenario, &unidade)
}

#[test]
fn testemunha_exata_preserva_faixa_estados_e_escape() {
    use gestao_marcadores::Marcador;

    let mut banco = Connection::open_in_memory().unwrap();
    banco
        .execute_batch(include_str!(
            "../../iesei/migrations/0043_gestao_marcadores.sql"
        ))
        .unwrap();
    let origem = Instalacao::interpretar("https://sei.test/sei").unwrap();
    let pagina = Pagina {
        total: 4,
        faixa: Some((2, 3)),
        itens: vec![
            Marcador {
                id: "01".into(),
                nome: "A & <B>".into(),
                cor: "Cor 'A'".into(),
                estado: EstadoMarcador::Ativo,
            },
            Marcador {
                id: "02".into(),
                nome: "A & <B>".into(),
                cor: "Cor \"B\"".into(),
                estado: EstadoMarcador::Inativo,
            },
        ],
    };
    gestao_marcadores::registrar(&mut banco, &origem, "a", "U", 1, &pagina).unwrap();
    let vazia = Pagina {
        total: 0,
        faixa: None,
        itens: vec![],
    };
    gestao_marcadores::registrar(&mut banco, &origem, "a", "U", 2, &vazia).unwrap();

    assert_eq!(
        responder_unidade(&banco, &origem, "a", "U").estado,
        Estado::DadosInsuficientes
    );
    assert!(gestao_marcadores::selecionar(&banco, &origem, "a", "U", 1).unwrap());
    let resposta = responder_unidade(&banco, &origem, "a", "U");
    assert_eq!(resposta.estado, Estado::Ok);
    assert!(
        resposta
            .corpo
            .contains("Lista de Marcadores (4 registros - 2 a 3):")
    );
    let linha = concat!(
        "<td>01</td><td>A &amp; &lt;B&gt;</td>",
        "<td><img title='Cor &#39;A&#39;'></td>",
        "<td><img title='Desativar Marcador'></td>"
    );
    assert!(resposta.corpo.contains(linha));
    let linha = concat!(
        "<td>02</td><td>A &amp; &lt;B&gt;</td>",
        "<td><img title='Cor &quot;B&quot;'></td>",
        "<td><img title='Reativar Marcador'></td>"
    );
    assert!(resposta.corpo.contains(linha));
    assert!(gestao_marcadores::selecionar(&banco, &origem, "a", "U", 2).unwrap());
    let resposta = responder_unidade(&banco, &origem, "a", "U");
    assert_eq!(resposta.estado, Estado::Ok);
    assert!(
        resposta
            .corpo
            .contains("Lista de Marcadores (0 registros):")
    );
    assert!(!resposta.corpo.contains("<td>"));
    for (cenario, unidade) in [("b", "U"), ("a", "V")] {
        assert_eq!(
            responder_unidade(&banco, &origem, cenario, unidade).estado,
            Estado::DadosInsuficientes
        );
    }
    assert_eq!(
        responder("POST", &banco, &origem, "a").estado,
        Estado::MetodoNaoPermitido
    );
}

#[test]
fn rota_e_menu_seguem_unidade_do_controle() {
    use iesei::banco_cassettes::{self, ControleResumido};
    use iesei::detalhada_cassettes::{self, ControleDetalhado};
    use iesei::observacao::{TabelaObservada, UnidadeExibida};

    let mut banco = Connection::open_in_memory().unwrap();
    banco_cassettes::iniciar(&banco).unwrap();
    banco
        .execute_batch(include_str!(
            "../../iesei/migrations/0043_gestao_marcadores.sql"
        ))
        .unwrap();
    let origem = Instalacao::interpretar("https://sei.test/sei").unwrap();
    let vazia = Pagina {
        total: 0,
        faixa: None,
        itens: vec![],
    };
    gestao_marcadores::registrar(&mut banco, &origem, "a", "U", 1, &vazia).unwrap();
    banco_cassettes::guardar(
        &mut banco,
        "a",
        &ControleResumido {
            instalacao: origem.clone(),
            unidade: UnidadeExibida::nova("U").unwrap(),
            recebidos: TabelaObservada::nova(Some(0), None, vec![]).unwrap(),
            gerados: TabelaObservada::nova(Some(0), None, vec![]).unwrap(),
        },
    )
    .unwrap();
    let acessar = |banco: &Connection, metodo, acao| {
        crate::rotas::responder(metodo, acao, b"", banco, &origem, "a")
    };
    let controle = "/sei/controlador.php?acao=procedimento_controlar";
    let lista = "/sei/controlador.php?acao=marcador_listar";
    assert!(
        !acessar(&banco, "GET", controle)
            .corpo
            .contains("acao=marcador_listar")
    );
    assert_eq!(
        acessar(&banco, "GET", lista).estado,
        Estado::DadosInsuficientes
    );
    assert!(gestao_marcadores::selecionar(&banco, &origem, "a", "U", 1).unwrap());
    assert!(
        acessar(&banco, "GET", controle)
            .corpo
            .contains("acao=marcador_listar")
    );
    assert_eq!(acessar(&banco, "GET", lista).estado, Estado::Ok);
    assert_eq!(
        acessar(&banco, "POST", lista).estado,
        Estado::MetodoNaoPermitido
    );

    detalhada_cassettes::guardar(
        &mut banco,
        "a",
        &ControleDetalhado {
            instalacao: origem.clone(),
            unidade: UnidadeExibida::nova("V").unwrap(),
            processos: TabelaObservada::nova(Some(0), None, vec![]).unwrap(),
        },
    )
    .unwrap();
    assert!(
        !acessar(&banco, "GET", controle)
            .corpo
            .contains("acao=marcador_listar")
    );
    assert_eq!(
        acessar(&banco, "GET", lista).estado,
        Estado::DadosInsuficientes
    );
    gestao_marcadores::registrar(&mut banco, &origem, "a", "V", 1, &vazia).unwrap();
    assert!(gestao_marcadores::selecionar(&banco, &origem, "a", "V", 1).unwrap());
    let resposta = acessar(&banco, "GET", lista);
    assert_eq!(resposta.estado, Estado::Ok);
    assert!(resposta.corpo.contains("id='lnkInfraUnidade'>V</a>"));
    assert!(
        acessar(&banco, "GET", controle)
            .corpo
            .contains("acao=marcador_listar")
    );
}
