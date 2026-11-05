//! Testes sintéticos do fluxo Web de unidades (issue #244).
use super::*;
use crate::formulario_web::RespostaDoPost;
use crate::transporte::RespostaWeb;
use std::io::Cursor;
const ORIGEM: &str = "https://sei.example/sei";
const HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
fn url(acao: &str) -> String {
    format!("{ORIGEM}/controlador.php?acao={acao}&infra_hash={HASH}")
}
fn resposta(html: String) -> RespostaDoPost<Cursor<Vec<u8>>> {
    RespostaDoPost {
        resposta: RespostaWeb {
            status_code: 200,
            mime: "text/html; charset=utf-8".to_owned(),
            content_length: None,
            localizacao: None,
            corpo: Cursor::new(html.into_bytes()),
        },
        localizacao: None,
    }
}
fn pagina(p: usize, ids: &[&str]) -> String {
    let linhas: String = ids
        .iter()
        .map(|id| {
            [
                "<tr><td><input type='checkbox' value='",
                id,
                "'></td><td>ABC/",
                id,
                "</td><td>Descrição</td><td>Órgão</td></tr>",
            ]
            .concat()
        })
        .collect();
    let acao = url("unidade_selecionar_envio_processo");
    format!(
        "<form id='frmUnidadeLista' method='post' action='{acao}'>
         <input type='hidden' name='hdnInfraPaginaAtual' value='{p}'>
         <select name='selOrgaoUnidade'><option value=''>Todos</option></select>
         <input name='txtSiglaUnidade' value=''>
         <input name='txtDescricaoUnidade' value=''>
         <select name='selInfraPaginacaoSuperior'>
           <option value='0'>1</option><option value='1'>2</option>
         </select>
         <select name='selInfraPaginacaoInferior'>
           <option value='0'>1</option><option value='1'>2</option>
         </select>
         <input type='submit' name='btnPesquisar' value='Pesquisar'>
         <table class='infraTable'><caption>Lista de Unidades (3 registros):</caption>
         <tr><th></th><th>Sigla</th><th>Descrição</th><th>Órgão</th></tr>
         {linhas}</table></form>"
    )
}
struct Fake {
    gets: usize,
    posts: usize,
    repetir: bool,
    filtro_sobreposto: bool,
}
impl TransporteLupa for Fake {
    type Corpo = Cursor<Vec<u8>>;
    fn get(
        &mut self,
        alvo: &str,
        _cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        let passo = self.gets;
        self.gets += 1;
        let (acao, html) = match passo {
            0 => (
                "procedimento_controlar",
                format!("<a href='{}'>Grupos</a>", url("grupo_unidade_listar")),
            ),
            1 => (
                "grupo_unidade_listar",
                format!(
                    "<button id='btnNovo' onclick=\"window.location.href='{}';\">Novo</button>",
                    url("grupo_unidade_cadastrar")
                ),
            ),
            2 => (
                "grupo_unidade_cadastrar",
                format!(
                    "<a href='{}'>Unidades</a>",
                    url("unidade_selecionar_envio_processo")
                ),
            ),
            3 => ("unidade_selecionar_envio_processo", pagina(0, &[])),
            _ => panic!("GET fora do orçamento"),
        };
        assert!(alvo.contains(&format!("acao={acao}")));
        Ok(resposta(html))
    }
    fn post(
        &mut self,
        envio: SubmissaoDerivada,
        _cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        assert_eq!(envio.url(), url("unidade_selecionar_envio_processo"));
        let indice_pagina = self.posts;
        assert!(
            String::from_utf8_lossy(envio.corpo())
                .contains(&format!("hdnInfraPaginaAtual={indice_pagina}"))
        );
        self.posts += 1;
        let ids: &[&str] = if indice_pagina == 0 {
            &["1", "2"]
        } else if self.repetir {
            &["1"]
        } else {
            &["3"]
        };
        let html = pagina(indice_pagina, ids);
        let html = if self.filtro_sobreposto {
            html.replace(
                "name='txtSiglaUnidade' value=''",
                "name='txtSiglaUnidade' value='FILTRADA'",
            )
        } else {
            html
        };
        Ok(resposta(html))
    }
}
#[test]
fn quatro_get_e_um_post_por_pagina_sem_repetir() {
    let mut rede = Fake {
        gets: 0,
        posts: 0,
        repetir: false,
        filtro_sobreposto: false,
    };
    let lida = ler_integral(
        &mut rede,
        &url("procedimento_controlar"),
        ORIGEM,
        "PHPSESSID=SINTETICO",
    )
    .unwrap();
    assert_eq!(lida.requisicoes, 6);
    assert_eq!(lida.total, 3);
    assert_eq!((rede.gets, rede.posts), (4, 2));
}
#[test]
fn filtro_restaurado_pelo_sei_nao_vira_lista_integral() {
    // Caso do Todos tests/test_destinos_indice_010.py @323eb01:
    // uma lista filtrada não substitui o snapshot integral.
    let mut rede = Fake {
        gets: 0,
        posts: 0,
        repetir: false,
        filtro_sobreposto: true,
    };
    assert_eq!(
        ler_integral(
            &mut rede,
            &url("procedimento_controlar"),
            ORIGEM,
            "PHPSESSID=SINTETICO",
        )
        .err(),
        Some(FalhaDaLupa::FormularioInesperado),
    );
    assert_eq!(
        (rede.gets, rede.posts),
        (4, 1),
        "falha fechada sem novo POST"
    );
}
#[test]
fn pagina_repetida_recusa_antes_de_persistir() {
    let mut rede = Fake {
        gets: 0,
        posts: 0,
        repetir: true,
        filtro_sobreposto: false,
    };
    let err = ler_integral(
        &mut rede,
        &url("procedimento_controlar"),
        ORIGEM,
        "PHPSESSID=SINTETICO",
    )
    .err()
    .unwrap();
    assert_eq!(err, FalhaDaLupa::Tela(ErroLupa::PaginasDivergentes));
    assert_eq!(rede.posts, 2);
}
