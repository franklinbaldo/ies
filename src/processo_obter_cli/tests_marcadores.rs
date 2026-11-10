//! RED/GREEN do JSON dos marcadores: **lista** com cada marcador (nunca
//! texto concatenado), total declarado e faixa; `null` com motivo, ou
//! ausente com `--sem-marcadores`; `nao_lidos` diz o que faltou.
use super::tests::aberto;
use super::*;
use crate::marcadores_processo::{MarcadorDoProcesso, MarcadoresLidos};
use crate::processo_obter::Marcadores;

fn nup() -> NumeroProcesso {
    NumeroProcesso::novo("00001.000001/2026-01").unwrap()
}

fn marcador(id: &str, nome: &str, cor: &str, texto: Option<&str>) -> MarcadorDoProcesso {
    MarcadorDoProcesso {
        id: id.into(),
        nome: nome.into(),
        cor: cor.into(),
        texto: texto.map(str::to_owned),
        usuario: Some("pessoa1".into()),
        data_hora: "03/11/2026 10:15".into(),
    }
}

fn com_marcadores(total: usize) -> ProcessoAberto {
    let mut aberto = aberto(Some("UNIDADE TESTE"));
    aberto.marcadores = Marcadores::Lidos(MarcadoresLidos {
        total_declarado: total,
        itens: vec![
            marcador("501", "Revis\u{e3}o", "azul", None),
            marcador("502", "Pendente", "amarelo", Some("Texto sint\u{e9}tico")),
        ],
    });
    aberto
}

#[test]
fn dois_marcadores_saem_como_lista_de_dois_objetos() {
    let valor = json_do_processo(&nup(), &com_marcadores(2), true);
    assert_eq!(
        valor["marcadores"],
        json!([
            {"id": "501", "rotulo": "501#Revis\u{e3}o", "nome": "Revis\u{e3}o", "cor": "azul", "texto": null,
             "usuario": "pessoa1", "data_hora": "03/11/2026 10:15"},
            {"id": "502", "rotulo": "502#Pendente", "nome": "Pendente", "cor": "amarelo", "texto": "Texto sint\u{e9}tico",
             "usuario": "pessoa1", "data_hora": "03/11/2026 10:15"}
        ])
    );
    assert_eq!(valor["marcadores_total_declarado"], 2);
    assert_eq!(
        valor["marcadores_faixa_observada"],
        json!({"inicio": 1, "fim": 2})
    );
    assert_eq!(valor["nao_lidos"], json!(["andamentos", "propriedades"]));
    assert!(valor.get("marcadores_nao_lidos_motivo").is_none());
}

#[test]
fn menos_linhas_que_a_legenda_publicam_total_e_faixa_sem_alegar_completude() {
    let valor = json_do_processo(&nup(), &com_marcadores(5), true);
    assert_eq!(valor["marcadores_total_declarado"], 5);
    assert_eq!(
        valor["marcadores_faixa_observada"],
        json!({"inicio": 1, "fim": 2})
    );
    let texto = humano(&nup(), &com_marcadores(5), true);
    assert!(texto.contains("Marcadores: 2 de 5 declarados"), "{texto}");
}

#[test]
fn nao_publicados_ou_sem_tabela_saem_null_com_motivo_e_omitidos_tiram_a_chave() {
    for (estado, motivo) in [
        (
            Marcadores::NaoPublicadosNaBarra,
            "tela_de_marcadores_nao_publicada_na_barra",
        ),
        (
            Marcadores::TabelaNaoPublicada,
            "tabela_de_marcadores_nao_publicada_na_tela",
        ),
    ] {
        let mut aberto = aberto(None);
        aberto.marcadores = estado;
        let valor = json_do_processo(&nup(), &aberto, true);
        assert!(valor.get("marcadores").is_some_and(Value::is_null));
        assert!(valor.get("marcadores_total_declarado").is_none());
        assert_eq!(valor["marcadores_nao_lidos_motivo"], motivo);
        assert_eq!(
            valor["nao_lidos"],
            json!(["andamentos", "marcadores", "propriedades"])
        );
    }
    let omitidos = json_do_processo(&nup(), &aberto(None), true);
    assert!(omitidos.get("marcadores").is_none());
    assert_eq!(
        omitidos["marcadores_nao_lidos_motivo"],
        "omitidos_por_sem_marcadores"
    );
}

#[test]
fn texto_humano_lista_cada_marcador_numa_linha() {
    let texto = humano(&nup(), &com_marcadores(2), true);
    assert!(texto.contains("Marcadores: 2 de 2 declarados:"), "{texto}");
    assert!(texto.contains("\n- 501#Revis\u{e3}o"), "{texto}");
    assert!(
        texto.contains("\n- 502#Pendente [amarelo]: Texto sint\u{e9}tico"),
        "{texto}"
    );
    let sem = humano(&nup(), &aberto(None), true);
    assert!(
        sem.contains("Marcadores não lidos (omitidos_por_sem_marcadores)."),
        "{sem}"
    );
}
