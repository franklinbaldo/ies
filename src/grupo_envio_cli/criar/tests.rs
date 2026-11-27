//! Transporte injetado sintético: não é replay de cassette nem anti-IES.
use super::*;
use crate::formulario_web::{RespostaDoPost, SubmissaoDerivada};
use crate::transporte::{FalhaTransporte, RespostaWeb, testes_http};
use std::collections::VecDeque;
use std::io::Cursor;

const ORIGEM: &str = "https://sei.test/sei";
const HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const UNIDADE: &str = "<a id='lnkInfraUnidade' href='#'>UNIDADE A</a>";

fn controle() -> String {
    format!(
        "<html><body>{UNIDADE}<a href='controlador.php?acao=grupo_unidade_listar&amp;infra_hash={HASH}'>Grupos</a>\
         <table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
         <table id='tblProcessosGerados'><caption>0 registros:</caption></table></body></html>"
    )
}

fn lista(id: &str) -> String {
    format!(
        "<html><body>{UNIDADE}<button id='btnNovo' onclick=\"location.href='controlador.php?acao=grupo_unidade_cadastrar&amp;infra_hash={HASH}'\">Novo</button>\
         <form id='frmGrupoUnidadeLista'><table><caption>Grupos de Envio (1 registro):</caption>\
         <tr><th></th><th>Nome</th><th>Acoes</th></tr>\
         <tr><td><input type='checkbox' value='{id}'></td><td>Grupo</td>\
         <td><a href='controlador.php?acao=grupo_unidade_consultar&amp;id_grupo_unidade={id}&amp;infra_hash={HASH}'>Consultar</a></td></tr>\
         </table></form></body></html>"
    )
}

fn cadastro() -> String {
    format!(
        "<html><body>{UNIDADE}<form id='frmGrupoCadastro' method='post' action='controlador.php?acao=grupo_unidade_cadastrar&amp;infra_hash={HASH}'>\
         <input type='hidden' name='hdnIdGrupoUnidade' value=''>\
         <input type='text' name='txtNome' value=''><textarea name='txaDescricao'></textarea>\
         <select name='selUnidades' multiple></select>\
         <input type='hidden' name='hdnUnidades' value=''>\
         <input type='submit' name='sbmCadastrarGrupoUnidade' value='Salvar'></form>\
         <script>new infraAjaxAutoCompletar('hdnIdUnidade','txtUnidade','controlador_ajax.php?acao_ajax=unidade_auto_completar_envio_processo&infra_hash={HASH}');</script></body></html>"
    )
}

fn consulta(nome: &str) -> String {
    format!(
        "<html><body>{UNIDADE}<form id='frmGrupoCadastro'><input name='hdnIdGrupoUnidade' value='001'>\
         <input name='txtNome' value='{nome}'><textarea name='txaDescricao'>Descrição</textarea>\
         <input name='hdnUnidades' value=''><select name='selUnidades'><option value='001'>X - Unidade</option></select></form></body></html>"
    )
}

struct Envio {
    telas: VecDeque<String>,
    mutacoes: usize,
    consultas: usize,
    location: &'static str,
}

impl EnvioDoControle for Envio {
    type Corpo = Cursor<Vec<u8>>;

    fn get(&mut self, _: &str, _: &str) -> Result<RespostaWeb<Self::Corpo>, FalhaTransporte> {
        Ok(testes_http::resposta(
            200,
            "text/html; charset=utf-8",
            self.telas.pop_front().expect("GET inesperado").into_bytes(),
            None,
        ))
    }

    fn post(
        &mut self,
        post: SubmissaoDerivada,
        _: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        if post.url().contains("controlador_ajax.php?") {
            self.consultas += 1;
            assert_eq!(post.corpo(), b"palavras_pesquisa=X");
            return Ok(testes_http::resposta_post(
                200,
                "application/xml; charset=utf-8",
                br#"<itens><item id="001" descricao="X - Unidade"></item></itens>"#.to_vec(),
                None,
                None,
            ));
        }
        assert!(post.url().contains("acao=grupo_unidade_cadastrar"));
        self.mutacoes += 1;
        assert_eq!(
            post.corpo(),
            b"hdnIdGrupoUnidade=&txtNome=Grupo&txaDescricao=Descri%C3%A7%C3%A3o&hdnUnidades=001%C2%B1X+-+Unidade&sbmCadastrarGrupoUnidade=Salvar"
        );
        Ok(testes_http::resposta_post(
            302, "text/html", Vec::new(), Some(0), Some(self.location.to_owned()),
        ))
    }
}

fn sessao() -> ReferenciasDaSessao {
    let pasta = std::env::temp_dir().join(format!(
        "ies-grupo-criar-{}-{}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
    ));
    std::fs::create_dir_all(&pasta).unwrap();
    let url = pasta.join("url");
    let origem = pasta.join("origem");
    let cookie = pasta.join("cookie");
    for (arquivo, valor) in [
        (&url, format!("{ORIGEM}/controlador.php?acao=procedimento_controlar&infra_hash={HASH}")),
        (&origem, ORIGEM.to_owned()),
        (&cookie, "PHPSESSID=SINTETICO".to_owned()),
    ] {
        std::fs::write(arquivo, valor).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(arquivo, std::fs::Permissions::from_mode(0o600)).unwrap();
        }
    }
    let sessao = ReferenciasDaSessao::ler(&url, &origem, &cookie).unwrap();
    std::fs::remove_dir_all(pasta).unwrap();
    sessao
}

#[test]
fn previa_e_confirmacao_distinguem_ajax_de_mutacao() {
    let sessao = sessao();
    let unidades = ["X".to_owned()];
    let mut pedido = Pedido { nome: "Grupo", descricao: "Descrição", unidades: &unidades, confirmar: false };
    let mut envio = Envio {
        telas: [controle(), lista("1"), cadastro()].into(),
        mutacoes: 0,
        consultas: 0,
        location: "controlador.php?acao=grupo_unidade_listar&id_grupo=001",
    };
    let previa = criar_via(&sessao, None, &pedido, &mut envio).unwrap();
    assert!(matches!(previa, Criacao::Previa { .. }));
    assert_eq!((envio.consultas, envio.mutacoes), (1, 0));
    assert!(envio.telas.is_empty());

    pedido.confirmar = true;
    envio.telas = [controle(), lista("1"), cadastro(), controle(), lista("001"), consulta("Grupo")].into();
    envio.consultas = 0;
    let criado = criar_via(&sessao, None, &pedido, &mut envio).unwrap();
    assert!(matches!(criado, Criacao::Criado { grupo, .. } if grupo.id == "001"));
    assert_eq!((envio.consultas, envio.mutacoes), (1, 1));
    assert!(envio.telas.is_empty());
}

#[test]
fn identidade_preexistente_ou_releitura_diferente_nao_confirma_nem_repete() {
    let sessao = sessao();
    let unidades = ["X".to_owned()];
    let pedido = Pedido { nome: "Grupo", descricao: "Descrição", unidades: &unidades, confirmar: true };
    for (antes, location, releitura) in [
        ("001", "controlador.php?acao=grupo_unidade_listar&id_grupo=001", None),
        ("1", "controlador.php?acao=grupo_unidade_listar#fragment&id_grupo=001", None),
        ("1", "controlador.php?acao=grupo_unidade_listar&id_grupo=001", Some("Outro")),
    ] {
        let mut telas = VecDeque::from([controle(), lista(antes), cadastro()]);
        if let Some(nome) = releitura {
            telas.extend([controle(), lista("001"), consulta(nome)]);
        }
        let mut envio = Envio { telas, mutacoes: 0, consultas: 0, location };
        assert!(criar_via(&sessao, None, &pedido, &mut envio).is_err());
        assert_eq!((envio.consultas, envio.mutacoes), (1, 1));
        assert!(envio.telas.is_empty());
    }
}
