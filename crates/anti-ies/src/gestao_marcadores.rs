//! Projeção local de uma testemunha explicitamente escolhida do catálogo de gestão.
//! WIP: a escolha do ordinal e a publicação no menu ainda não estão ligadas à rota HTTP.
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

fn responder(
    metodo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    unidade: &str,
    ordinal: usize,
) -> Resposta {
    if metodo != "GET" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    match gestao_marcadores::ler(banco, origem, cenario, unidade, ordinal) {
        Ok(Some(pagina)) => Resposta::html(html(unidade, &pagina)),
        Ok(None) | Err(_) => Resposta::nova(Estado::DadosInsuficientes),
    }
}

#[test]
fn testemunha_exata_preserva_faixa_estados_e_escape() {
    use gestao_marcadores::Marcador;

    let mut banco = Connection::open_in_memory().unwrap();
    banco
        .execute_batch(include_str!("../../iesei/migrations/0043_gestao_marcadores.sql"))
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

    let resposta = responder("GET", &banco, &origem, "a", "U", 1);
    assert_eq!(resposta.estado, Estado::Ok);
    assert!(resposta.corpo.contains("Lista de Marcadores (4 registros - 2 a 3):"));
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
    let resposta = responder("GET", &banco, &origem, "a", "U", 2);
    assert_eq!(resposta.estado, Estado::Ok);
    assert!(resposta.corpo.contains("Lista de Marcadores (0 registros):"));
    assert!(!resposta.corpo.contains("<td>"));
    for (cenario, unidade, ordinal) in [("a", "U", 3), ("b", "U", 1), ("a", "V", 1)] {
        assert_eq!(
            responder("GET", &banco, &origem, cenario, unidade, ordinal).estado,
            Estado::DadosInsuficientes
        );
    }
    assert_eq!(
        responder("POST", &banco, &origem, "a", "U", 1).estado,
        Estado::MetodoNaoPermitido
    );
}
