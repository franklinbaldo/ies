use super::*;
use crate::formulario_web::{RespostaDoPost, SubmissaoDerivada};
use crate::transporte::RespostaWeb;
use std::collections::VecDeque;
use std::io::Cursor;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const ORIGEM: &str = "https://sei.exemplo.gov.br/sei";
struct Arquivos {
    pasta: PathBuf,
    sessao: ReferenciasDaSessao,
}
impl Arquivos {
    fn criar() -> Self {
        let pasta = std::env::temp_dir().join(format!(
            "ies-bloco-teste-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&pasta).unwrap();
        let arquivos = [
            (
                "controle",
                format!("{ORIGEM}/controlador.php?acao=procedimento_controlar&infra_hash={HASH}"),
            ),
            ("origem", ORIGEM.to_owned()),
            ("cookie", "PHPSESSID=SINTETICO".to_owned()),
        ];
        for (nome, texto) in arquivos {
            let caminho = pasta.join(nome);
            std::fs::write(&caminho, texto).unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&caminho, std::fs::Permissions::from_mode(0o600)).unwrap();
            }
        }
        let sessao = ReferenciasDaSessao::ler(
            &pasta.join("controle"),
            &pasta.join("origem"),
            &pasta.join("cookie"),
        )
        .unwrap();
        Self { pasta, sessao }
    }
}
impl Drop for Arquivos {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.pasta).unwrap();
    }
}
struct EnvioFalso {
    telas: VecDeque<(String, String)>,
    urls: Vec<String>,
    aceite: Option<(i32, Option<String>)>,
    posts: usize,
}
impl EnvioDoControle for EnvioFalso {
    type Corpo = Cursor<Vec<u8>>;
    fn post(
        &mut self,
        submissao: SubmissaoDerivada,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        let (status_code, localizacao) = self.aceite.take().expect("POST não previsto ou repetido");
        assert_eq!(self.urls.len(), 3, "POST antes da prévia completa");
        assert_eq!(cookie, "PHPSESSID=SINTETICO");
        assert_eq!(
            submissao.url(),
            format!("{ORIGEM}/controlador.php?acao=bloco_assinatura_cadastrar&infra_hash={HASH}")
        );
        assert_eq!(
            submissao.corpo(),
            b"txtDescricao=Treinamento&selGrupoBloco=null&sbmCadastrarBloco=Salvar"
        );
        self.posts += 1;
        Ok(RespostaDoPost {
            resposta: RespostaWeb {
                status_code,
                mime: "text/html; charset=utf-8".to_owned(),
                content_length: None,
                localizacao: None,
                corpo: Cursor::new(Vec::new()),
            },
            localizacao,
        })
    }
    fn get(
        &mut self,
        url: &str,
        cookie: &str,
    ) -> Result<RespostaWeb<Self::Corpo>, FalhaTransporte> {
        assert_eq!(cookie, "PHPSESSID=SINTETICO");
        let (esperada, html) = self.telas.pop_front().expect("GET não previsto");
        assert_eq!(url, esperada);
        self.urls.push(url.to_owned());
        Ok(RespostaWeb {
            status_code: 200,
            mime: "text/html; charset=utf-8".to_owned(),
            content_length: None,
            localizacao: None,
            corpo: Cursor::new(html.into_bytes()),
        })
    }
}
fn envio(detalhe: bool) -> EnvioFalso {
    let controle = format!(
        "<html><a id='lnkInfraUnidade'>PPI</a><input id='hdnTipoVisualizacao' value='R'>\
         <table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
         <table id='tblProcessosGerados'><caption>0 registros:</caption></table>\
         <a href='controlador.php?acao=bloco_assinatura_listar&amp;infra_hash={HASH}'>Blocos</a></html>"
    );
    let lista = format!(
        "<table id='tblBlocos'><caption>Lista (1 registros):</caption>\
         <tr><th></th><th>Número</th><th>Estado</th><th>Descrição</th></tr>\
         <tr><td><input type='checkbox'></td><td><a href='controlador.php?acao=rel_bloco_protocolo_listar&amp;id_bloco=2195&amp;infra_hash={HASH}'>2195</a></td>\
         <td>Gerado</td><td>Treinamento</td></tr></table>"
    );
    let mut telas = VecDeque::from([
        (
            format!("{ORIGEM}/controlador.php?acao=procedimento_controlar&infra_hash={HASH}"),
            controle,
        ),
        (
            format!("{ORIGEM}/controlador.php?acao=bloco_assinatura_listar&infra_hash={HASH}"),
            lista,
        ),
    ]);
    if detalhe {
        let html = "<table id='tblProtocolosBlocos'><caption>Lista (1 registros):</caption>\
            <tr><th></th><th>Processo</th><th>Documento</th><th>Anotações</th></tr>\
            <tr><td><input type='checkbox'></td><td>NUP</td><td>001</td><td></td></tr></table>";
        telas.push_back((
            format!("{ORIGEM}/controlador.php?acao=rel_bloco_protocolo_listar&id_bloco=2195&infra_hash={HASH}"),
            html.to_owned(),
        ));
    }
    EnvioFalso {
        telas,
        urls: Vec::new(),
        aceite: None,
        posts: 0,
    }
}
#[test]
fn lista_sao_exatamente_dois_get_sem_post() {
    // Caso de Todos tests/paginas_reais/menu_blocos.html @323eb01.
    let arquivos = Arquivos::criar();
    let mut envio = envio(false);
    let valor = ler_via(
        &arquivos.sessao,
        Some("PPI"),
        TipoBloco::Assinatura,
        None,
        None,
        &mut envio,
    )
    .unwrap();
    assert_eq!(envio.urls.len(), 2);
    assert!(envio.telas.is_empty());
    assert_eq!(valor["requisicoes"], 2);
    assert!(valor["lista"]["itens"][0].get("quantidade_itens").is_none());
    assert!(!valor.to_string().contains(HASH));
    assert!(!valor.to_string().contains("PHPSESSID"));
}
#[test]
fn detalhe_sao_tres_get_sem_post_e_so_numero_da_linha() {
    // Caso de Todos tests/paginas_reais/bloco_com_documentos.html @323eb01.
    let arquivos = Arquivos::criar();
    let mut envio = envio(true);
    let valor = ler_via(
        &arquivos.sessao,
        Some("PPI"),
        TipoBloco::Assinatura,
        Some("2195"),
        None,
        &mut envio,
    )
    .unwrap();
    assert_eq!(envio.urls.len(), 3);
    assert!(envio.telas.is_empty());
    assert_eq!(valor["requisicoes"], 3);
    assert_eq!(valor["bloco"]["itens"][0]["documento"], "001");
}
#[test]
fn cadastro_post_unico_e_releitura_independente() {
    let arquivos = Arquivos::criar();
    let casos = [
        (302, Some("2196"), true, false),
        (302, Some("2195"), false, false),
        (302, Some("9999"), false, false),
        (302, None, false, false),
        (500, None, false, false),
        (302, Some("2196"), false, true),
    ];
    for (status, id, confirmado, mudou_unidade) in casos {
        let mut envio = envio(false);
        let controle = envio.telas.front().unwrap().clone();
        let lista = envio.telas.back_mut().unwrap();
        let depois = (
            lista.0.clone(),
            lista
                .1
                .replace("1 registros", "2 registros")
                .replace(
                    "</table>",
                    &format!(
                        "<tr><td><input type='checkbox'></td><td><a href='controlador.php?acao=rel_bloco_protocolo_listar&amp;id_bloco=2196&amp;infra_hash={HASH}'>2196</a></td><td>Gerado</td><td>Treinamento</td></tr></table>"
                    ),
                ),
        );
        let cadastro = format!("controlador.php?acao=bloco_assinatura_cadastrar&infra_hash={HASH}");
        lista.1.push_str(&format!(
            "<button id='btnNovo' onclick=\"location.href='{cadastro}'\">Novo</button>"
        ));
        envio.telas.push_back((
            format!("{ORIGEM}/{cadastro}"),
            format!(
                "<form id='frmBlocoCadastro' method='post' action='{cadastro}'>\
                 <textarea name='txtDescricao'></textarea>\
                 <select name='selGrupoBloco'><option value='null'>Nenhum</option></select>\
                 <button type='submit' name='sbmCadastrarBloco' value='Salvar'>Salvar</button></form>"
            ),
        ));
        if status == 302 {
            if mudou_unidade {
                envio
                    .telas
                    .push_back((controle.0, controle.1.replace(">PPI<", ">OUTRA<")));
            } else {
                envio.telas.push_back(controle);
                envio.telas.push_back(depois);
            }
        }
        envio.aceite = Some((
            status,
            id.map(|id| format!("controlador.php?acao=bloco_assinatura_listar&id_bloco={id}")),
        ));
        let resultado = criar::criar_via(
            &arquivos.sessao,
            Some("PPI"),
            "Treinamento",
            true,
            &mut envio,
        );
        assert_eq!(envio.posts, 1);
        assert!(envio.telas.is_empty());
        let gets_esperados = match (status, mudou_unidade) {
            (302, true) => 4,
            (302, false) => 5,
            _ => 3,
        };
        assert_eq!(envio.urls.len(), gets_esperados);
        if confirmado {
            let valor = resultado.unwrap();
            assert_eq!(valor["bloco"]["numero"], "2196");
            assert_eq!(valor["confirmado"], true);
        } else {
            assert!(matches!(resultado, Err(FalhaLeitura::NaoConfirmado)));
        }
    }
}

#[test]
fn unidade_diferente_interrompe_antes_do_link_de_bloco() {
    let arquivos = Arquivos::criar();
    let mut envio = envio(false);
    let erro = ler_via(
        &arquivos.sessao,
        Some("ERRADA"),
        TipoBloco::Assinatura,
        None,
        None,
        &mut envio,
    )
    .err()
    .unwrap();
    assert!(matches!(erro, FalhaLeitura::UnidadeInesperada));
    assert_eq!(envio.urls.len(), 1);
}
