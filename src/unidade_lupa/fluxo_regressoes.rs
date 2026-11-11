//! Regressões de fronteira da leitura Web da lupa (#244).
//! Todas as URLs, hashes e respostas HTTP deste arquivo são sintéticos.
use super::*;
use crate::acao_do_sei::AcaoDoSei;

const ORIGEM: &str = "https://sei.example/sei";
const HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn link(query: &str) -> String {
    format!("<a href='https://sei.example/sei/controlador.php?{query}'>Unidades</a>")
}

fn controle() -> String {
    format!("{ORIGEM}/controlador.php?acao=procedimento_controlar&infra_hash={HASH}")
}

#[test]
fn link_publicado_para_outra_acao_pode_ser_lido_com_hash_valido() {
    let consulta = format!("acao=grupo_unidade_listar&infra_hash={HASH}");
    assert_eq!(
        destino(
            &link(&consulta),
            &controle(),
            ORIGEM,
            AcaoDoSei::GrupoUnidadeListar
        ),
        Ok(format!("{ORIGEM}/controlador.php?{consulta}"))
    );
}

#[test]
fn link_com_hash_repetido_valido_ou_invalido_eh_recusado() {
    for adicional in [HASH, "invalido", ""] {
        let consulta =
            format!("acao=grupo_unidade_listar&infra_hash={HASH}&infra_hash={adicional}");
        assert_eq!(
            destino(
                &link(&consulta),
                &controle(),
                ORIGEM,
                AcaoDoSei::GrupoUnidadeListar
            ),
            Err(FalhaDaLupa::LinkAusente),
        );
    }
}

#[test]
fn consulta_malformada_ou_com_chave_nao_canonica_eh_recusada() {
    let exemplos = [
        format!("acao=grupo_unidade_listar&infra_hash={HASH}&sem_igual"),
        format!("acao=grupo_unidade_listar&infra_hash={HASH}&Acao=x"),
        format!("acao=grupo_unidade_listar&infra_hash={HASH}&outra=x;y"),
        format!("acao=grupo_unidade_listar&infra_hash={HASH}&acao=outra"),
    ];
    for consulta in exemplos {
        assert_eq!(
            destino(
                &link(&consulta),
                &controle(),
                ORIGEM,
                AcaoDoSei::GrupoUnidadeListar
            ),
            Err(FalhaDaLupa::LinkAusente),
            "uma consulta malformada nunca deve autorizar a leitura"
        );
    }
}

#[test]
fn link_assinado_ambiguo_nao_dispara_segunda_requisicao() {
    use crate::formulario_web::RespostaDoPost;
    use crate::transporte::RespostaWeb;
    use std::io::Cursor;

    struct UnicaLeitura {
        gets: usize,
    }
    impl TransporteLupa for UnicaLeitura {
        type Corpo = Cursor<Vec<u8>>;
        fn get(
            &mut self,
            _url: &str,
            _cookie: &str,
        ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
            self.gets += 1;
            assert_eq!(self.gets, 1, "um link inseguro nao pode causar outro GET");
            let consulta =
                format!("acao=grupo_unidade_listar&infra_hash={HASH}&infra_hash=invalido");
            let resposta = RespostaWeb {
                status_code: 200,
                mime: "text/html; charset=utf-8".to_owned(),
                content_length: None,
                localizacao: None,
                corpo: Cursor::new(link(&consulta).into_bytes()),
            };
            Ok(RespostaDoPost {
                resposta,
                localizacao: None,
            })
        }
        fn post(
            &mut self,
            _submissao: SubmissaoDerivada,
            _cookie: &str,
        ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
            panic!("nunca POST antes de conferir o link");
        }
    }

    let mut transporte = UnicaLeitura { gets: 0 };
    let mut requisicoes = 0;
    assert!(matches!(
        ler_integral_com_contador(
            &mut transporte,
            &controle(),
            ORIGEM,
            "PHPSESSID=SINTETICO",
            &mut requisicoes,
        ),
        Err(FalhaDaLupa::LinkAusente),
    ));
    assert_eq!(transporte.gets, 1);
    assert_eq!(requisicoes, 1, "um GET servido e nenhum POST");
}

#[test]
fn btn_novo_sem_destino_nao_herda_link_de_outro_controle() {
    // Regressão da revisão #250: apenas o próprio btnNovo autoriza abrir cadastro.
    let cadastro = format!("acao=grupo_unidade_cadastrar&infra_hash={HASH}");
    let html = format!(
        "<button id='btnNovo'>Novo</button><a href='https://sei.example/sei/controlador.php?{cadastro}'>Outro controle</a>"
    );
    assert_eq!(
        botao_novo(&html, &controle(), ORIGEM),
        Err(FalhaDaLupa::LinkAusente)
    );
}

#[test]
fn btn_novo_nao_eh_ambiguo_por_link_alheio() {
    let cadastro = format!("acao=grupo_unidade_cadastrar&infra_hash={HASH}");
    let outro_hash = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let outro = format!("acao=grupo_unidade_cadastrar&infra_hash={outro_hash}");
    let html = format!(
        "<button id='btnNovo' onclick=\"window.location.href='https://sei.example/sei/controlador.php?{cadastro}';\">Novo</button><a href='https://sei.example/sei/controlador.php?{outro}'>Outro controle</a>"
    );
    assert_eq!(
        botao_novo(&html, &controle(), ORIGEM),
        Ok(format!("{ORIGEM}/controlador.php?{cadastro}"))
    );
}

#[test]
fn botao_novo_repetido_eh_recusado_sem_get() {
    let consulta = format!("acao=grupo_unidade_cadastrar&infra_hash={HASH}");
    let html = format!(
        "<button id='btnNovo' onclick=\"window.location.href='https://sei.example/sei/controlador.php?{consulta}';\">Novo</button><button id='btnNovo'>Novo duplicado</button>"
    );
    assert_eq!(
        botao_novo(&html, &controle(), ORIGEM),
        Err(FalhaDaLupa::LinkAmbiguo)
    );
}

#[test]
fn btn_novo_repetido_com_mesmo_alvo_observado_na_unb_eh_unico() {
    // Caso do acervo UnB 57497bc11c4f: dois btnNovo idênticos.
    let consulta = format!("acao=grupo_unidade_cadastrar&infra_hash={HASH}");
    let botao = format!(
        "<button id='btnNovo' onclick=\"window.location.href='https://sei.example/sei/controlador.php?{consulta}';\">Novo</button>"
    );
    assert_eq!(
        botao_novo(&format!("{botao}{botao}"), &controle(), ORIGEM),
        Ok(format!("{ORIGEM}/controlador.php?{consulta}"))
    );
}

#[test]
fn btn_novo_repetido_com_alvos_assinados_distintos_falha_fechado() {
    let outro_hash = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let primeiro = format!("acao=grupo_unidade_cadastrar&infra_hash={HASH}");
    let segundo = format!("acao=grupo_unidade_cadastrar&infra_hash={outro_hash}");
    let html = format!(
        "<button id='btnNovo' onclick=\"window.location.href='https://sei.example/sei/controlador.php?{primeiro}';\">Novo</button>\
         <button id='btnNovo' onclick=\"window.location.href='https://sei.example/sei/controlador.php?{segundo}';\">Novo</button>"
    );
    assert_eq!(
        botao_novo(&html, &controle(), ORIGEM),
        Err(FalhaDaLupa::LinkAmbiguo),
        "sem escolher hash arbitrário e sem requisitar a Web"
    );
}

#[test]
fn lupa_go_publicada_como_literal_de_script_eh_lida_sem_executar_js() {
    // Observação estrutural: cassette GO unidade-listar-destinos-go,
    // grupo_unidade_cadastrar (passos 2, 6, 10): a URL da lupa aparece
    // no HTML, mas não em href/onclick de a, button ou input.
    let consulta = format!("acao=unidade_selecionar_envio_processo&infra_hash={HASH}");
    let html = format!("<script>const lupa = 'controlador.php?{consulta}';</script>");
    assert_eq!(
        destino(
            &html,
            &controle(),
            ORIGEM,
            AcaoDoSei::UnidadeSelecionarEnvioProcesso
        ),
        Ok(format!("{ORIGEM}/controlador.php?{consulta}")),
    );
}

#[test]
fn dois_literals_assinados_distintos_da_lupa_sao_ambiguos() {
    let outro = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let html = format!(
        "<script>const a='controlador.php?acao=unidade_selecionar_envio_processo&infra_hash={HASH}'; const b='controlador.php?acao=unidade_selecionar_envio_processo&infra_hash={outro}';</script>"
    );
    assert_eq!(
        destino(
            &html,
            &controle(),
            ORIGEM,
            AcaoDoSei::UnidadeSelecionarEnvioProcesso
        ),
        Err(FalhaDaLupa::LinkAmbiguo),
    );
}

#[test]
fn literal_sem_hash_assinado_nao_autoriza_get() {
    let html =
        "<script>const lupa='controlador.php?acao=unidade_selecionar_envio_processo';</script>";
    assert_eq!(
        destino(
            html,
            &controle(),
            ORIGEM,
            AcaoDoSei::UnidadeSelecionarEnvioProcesso
        ),
        Err(FalhaDaLupa::LinkAusente),
    );
}

#[test]
fn transporte_recusado_conta_tentativa_sem_retry() {
    use crate::formulario_web::RespostaDoPost;
    use std::io::Cursor;

    struct Recusa {
        gets: usize,
    }

    impl TransporteLupa for Recusa {
        type Corpo = Cursor<Vec<u8>>;

        fn get(
            &mut self,
            _url: &str,
            _cookie: &str,
        ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
            self.gets += 1;
            Err(FalhaTransporte::Status(
                crate::transporte::StatusHttpRecusado {
                    status: 503,
                    mensagem_do_servidor: Some("serviço indisponível".to_owned()),
                },
            ))
        }

        fn post(
            &mut self,
            _envio: SubmissaoDerivada,
            _cookie: &str,
        ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
            panic!("sem POST na falha do primeiro GET")
        }
    }

    let mut rede = Recusa { gets: 0 };
    let mut requisicoes = 0;
    let falha = ler_integral_com_contador(
        &mut rede,
        &controle(),
        ORIGEM,
        "PHPSESSID=SINTETICO",
        &mut requisicoes,
    )
    .err()
    .expect("HTTP 503 deve falhar");
    assert_eq!(
        falha,
        FalhaDaLupa::Transporte(FalhaTransporte::Status(
            crate::transporte::StatusHttpRecusado {
                status: 503,
                mensagem_do_servidor: Some("serviço indisponível".to_owned()),
            }
        ))
    );
    assert_eq!(requisicoes, 1);
    assert_eq!(rede.gets, 1, "sem retry da tentativa recusada");
}
