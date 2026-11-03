//! Abertura, permissões, falhas que desligam, concorrência, identidade e
//! capacidade da instalação. Diretórios temporários, sem rede.
use super::*;
use crate::instalacao::Instalacao;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

pub(super) fn pasta(nome: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "ies-indice-{nome}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir(&dir).unwrap();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
    dir
}

pub(super) fn instalacao() -> Instalacao {
    Instalacao::interpretar("https://sei.exemplo/sei").unwrap()
}

fn modo(caminho: &Path) -> u32 {
    fs::metadata(caminho).unwrap().permissions().mode() & 0o777
}

#[test]
fn cria_diretorio_0700_e_banco_0600_com_wal_herdando() {
    let raiz = pasta("permissoes");
    let caminho = raiz.join("sub").join(NOME_DO_ARQUIVO);
    let indice = IndiceLocal::em(caminho.clone());
    indice
        .registrar_processo(&instalacao(), "N1", "10")
        .unwrap();
    assert!(indice.ligado());
    assert_eq!(
        indice.id_por_nup(&instalacao(), "N1").as_deref(),
        Some("10")
    );
    assert_eq!(modo(&raiz.join("sub")), 0o700);
    assert_eq!(modo(&caminho), 0o600);
    for sufixo in ["-wal", "-shm"] {
        let lateral = PathBuf::from(format!("{}{sufixo}", caminho.display()));
        if lateral.exists() {
            assert_eq!(modo(&lateral), 0o600, "{sufixo}");
        }
    }
}

#[test]
fn banco_link_simbolico_desliga_sem_tocar_no_alvo() {
    let raiz = pasta("symlink");
    let alvo = raiz.join("alvo");
    fs::write(&alvo, b"").unwrap();
    fs::set_permissions(&alvo, fs::Permissions::from_mode(0o600)).unwrap();
    let caminho = raiz.join(NOME_DO_ARQUIVO);
    std::os::unix::fs::symlink(&alvo, &caminho).unwrap();
    let indice = IndiceLocal::em(caminho);
    let _ = indice.registrar_processo(&instalacao(), "N1", "10");
    assert_eq!(indice.id_por_nup(&instalacao(), "N1"), None);
    assert!(!indice.ligado());
    assert_eq!(fs::metadata(&alvo).unwrap().len(), 0);
}

#[test]
fn wal_link_simbolico_desliga() {
    let raiz = pasta("wal-symlink");
    let caminho = raiz.join(NOME_DO_ARQUIVO);
    IndiceLocal::em(caminho.clone()).registrar_entrada_por_id(&instalacao(), true);
    let wal = PathBuf::from(format!("{}-wal", caminho.display()));
    let _ = fs::remove_file(&wal);
    std::os::unix::fs::symlink(raiz.join("outro"), &wal).unwrap();
    let indice = IndiceLocal::em(caminho);
    assert_eq!(indice.entrada_por_id(&instalacao()), None);
    assert!(!indice.ligado());
}

#[test]
fn diretorio_aberto_a_outros_desliga_e_nao_cria_banco() {
    let raiz = pasta("dir-aberto");
    fs::set_permissions(&raiz, fs::Permissions::from_mode(0o755)).unwrap();
    let caminho = raiz.join(NOME_DO_ARQUIVO);
    let indice = IndiceLocal::em(caminho.clone());
    indice.registrar_entrada_por_id(&instalacao(), true);
    assert!(!indice.ligado());
    assert!(!caminho.exists());
}

#[test]
fn banco_existente_com_bits_de_grupo_desliga() {
    let raiz = pasta("banco-aberto");
    let caminho = raiz.join(NOME_DO_ARQUIVO);
    IndiceLocal::em(caminho.clone()).registrar_entrada_por_id(&instalacao(), true);
    fs::set_permissions(&caminho, fs::Permissions::from_mode(0o640)).unwrap();
    let indice = IndiceLocal::em(caminho);
    assert_eq!(indice.entrada_por_id(&instalacao()), None);
    assert!(!indice.ligado());
}

#[test]
fn caminho_respeita_sem_indice_arquivo_e_padrao() {
    let url = Path::new("/privado/ies/url-controle");
    let padrao = Some(PathBuf::from("/privado/ies/ies.sqlite3"));
    let os = |s: &'static str| Some(OsStr::new(s));
    assert_eq!(caminho_do_indice(None, None, url), padrao);
    assert_eq!(caminho_do_indice(os(""), None, url), padrao);
    assert_eq!(caminho_do_indice(os("0"), os(""), url), padrao);
    assert_eq!(caminho_do_indice(os("1"), None, url), None);
    assert_eq!(caminho_do_indice(os("sim"), os("/x/i.db"), url), None);
    assert_eq!(
        caminho_do_indice(None, os("/x/i.db"), url),
        Some(PathBuf::from("/x/i.db"))
    );
}

#[test]
fn desligado_nao_cria_nada_e_nao_responde() {
    let indice = IndiceLocal::desligado();
    indice.registrar_entrada_por_id(&instalacao(), true);
    indice
        .registrar_processo(&instalacao(), "N1", "10")
        .unwrap();
    assert!(!indice.ligado());
    assert_eq!(indice.entrada_por_id(&instalacao()), None);
    assert_eq!(indice.id_por_nup(&instalacao(), "N1"), None);
}

#[test]
fn arquivo_corrompido_desliga_sem_erro_e_sem_apagar() {
    let raiz = pasta("corrompido");
    let caminho = raiz.join(NOME_DO_ARQUIVO);
    let lixo = vec![b'x'; 8192];
    fs::write(&caminho, &lixo).unwrap();
    fs::set_permissions(&caminho, fs::Permissions::from_mode(0o600)).unwrap();
    let indice = IndiceLocal::em(caminho.clone());
    indice.registrar_entrada_por_id(&instalacao(), true);
    assert_eq!(indice.entrada_por_id(&instalacao()), None);
    assert!(!indice.ligado());
    assert_eq!(fs::read(&caminho).unwrap(), lixo);
}

#[test]
fn versao_futura_desliga_sem_migrar() {
    let raiz = pasta("versao-futura");
    let caminho = raiz.join(NOME_DO_ARQUIVO);
    IndiceLocal::em(caminho.clone()).registrar_entrada_por_id(&instalacao(), true);
    let conexao = rusqlite::Connection::open(&caminho).unwrap();
    conexao.pragma_update(None, "user_version", 99).unwrap();
    drop(conexao);
    let indice = IndiceLocal::em(caminho.clone());
    assert_eq!(indice.entrada_por_id(&instalacao()), None);
    assert!(!indice.ligado());
    let conexao = rusqlite::Connection::open(&caminho).unwrap();
    let versao: i64 = conexao
        .pragma_query_value(None, "user_version", |l| l.get(0))
        .unwrap();
    assert_eq!(versao, 99);
}

#[test]
fn esquema_versionado_em_wal() {
    let raiz = pasta("esquema");
    let caminho = raiz.join(NOME_DO_ARQUIVO);
    IndiceLocal::em(caminho.clone()).registrar_entrada_por_id(&instalacao(), true);
    let conexao = rusqlite::Connection::open(&caminho).unwrap();
    let versao: i64 = conexao
        .pragma_query_value(None, "user_version", |l| l.get(0))
        .unwrap();
    let modo: String = conexao
        .pragma_query_value(None, "journal_mode", |l| l.get(0))
        .unwrap();
    assert_eq!((versao, modo.as_str()), (1, "wal"));
}

#[test]
fn duas_conexoes_concorrentes_gravam_sem_perder_linhas() {
    let raiz = pasta("concorrente");
    let caminho = raiz.join(NOME_DO_ARQUIVO);
    let tarefas: Vec<_> = ["A", "B"]
        .into_iter()
        .enumerate()
        .map(|(n, prefixo)| {
            let caminho = caminho.clone();
            std::thread::spawn(move || {
                let indice = IndiceLocal::em(caminho);
                for i in 0..40 {
                    let id = (n * 1000 + i).to_string();
                    indice
                        .registrar_processo(&instalacao(), &format!("{prefixo}{i}"), &id)
                        .unwrap();
                }
                indice.ligado()
            })
        })
        .collect();
    for tarefa in tarefas {
        assert!(tarefa.join().unwrap());
    }
    let leitor = IndiceLocal::em(caminho);
    for i in 0..40 {
        assert_eq!(
            leitor.id_por_nup(&instalacao(), &format!("A{i}")),
            Some(i.to_string())
        );
        let b = (1000 + i).to_string();
        assert_eq!(leitor.id_por_nup(&instalacao(), &format!("B{i}")), Some(b));
    }
}

#[test]
fn conflito_substitui_as_linhas_antigas() {
    let raiz = pasta("conflito");
    let indice = IndiceLocal::em(raiz.join(NOME_DO_ARQUIVO));
    let sei = instalacao();
    indice.registrar_processo(&sei, "A", "1").unwrap();
    indice.registrar_processo(&sei, "B", "2").unwrap();
    indice.registrar_processo(&sei, "A", "2").unwrap();
    assert_eq!(indice.id_por_nup(&sei, "A").as_deref(), Some("2"));
    assert_eq!(indice.id_por_nup(&sei, "B"), None);
    indice.registrar_processo(&sei, "C", "2").unwrap();
    assert_eq!(indice.id_por_nup(&sei, "A"), None);
    assert_eq!(indice.id_por_nup(&sei, "C").as_deref(), Some("2"));
    indice.esquecer_processo(&sei, "C");
    assert_eq!(indice.id_por_nup(&sei, "C"), None);
    assert!(indice.ligado());
}

#[test]
fn nup_opaco_comparado_exatamente_e_por_instalacao() {
    let raiz = pasta("nup-opaco");
    let indice = IndiceLocal::em(raiz.join(NOME_DO_ARQUIVO));
    let sei = instalacao();
    let outra = Instalacao::interpretar("https://sei.exemplo/outro").unwrap();
    indice
        .registrar_processo(&sei, "23106.000001/2026-01", "7")
        .unwrap();
    indice
        .registrar_processo(&outra, "23106.000001/2026-01", "7")
        .unwrap();
    assert_eq!(indice.id_por_nup(&sei, "23106000001202601"), None);
    assert_eq!(indice.id_por_nup(&sei, "23106.000001/2026-01 "), None);
    indice.esquecer_processo(&outra, "23106.000001/2026-01");
    assert_eq!(
        indice.id_por_nup(&sei, "23106.000001/2026-01").as_deref(),
        Some("7")
    );
}

#[test]
fn identidade_invalida_recusada_sem_gravar() {
    let raiz = pasta("identidade-invalida");
    let indice = IndiceLocal::em(raiz.join(NOME_DO_ARQUIVO));
    let sei = instalacao();
    let r = RegistroRecusado::IdProcedimentoInvalido;
    for id in ["", "12a", "1&acao=x", "-1", "123456789012345678901"] {
        assert_eq!(indice.registrar_processo(&sei, "N", id), Err(r), "{id}");
    }
    for nup in ["", "a\nb", "a\u{0}b"] {
        assert_eq!(
            indice.registrar_processo(&sei, nup, "1"),
            Err(RegistroRecusado::NupInvalido)
        );
    }
    assert_eq!(indice.id_por_nup(&sei, "N"), None);
    assert!(indice.ligado());
}

fn relogio() -> (Arc<AtomicU64>, impl Fn() -> f64 + Send + 'static) {
    let agora = Arc::new(AtomicU64::new(1_800_000_000));
    let leitura = Arc::clone(&agora);
    #[allow(clippy::cast_precision_loss)]
    let ler = move || leitura.load(Ordering::SeqCst) as f64;
    (agora, ler)
}

const DIA: u64 = 24 * 3600;

#[test]
fn recusa_de_desconhecida_vale_na_primeira_e_expira_em_30_dias() {
    let raiz = pasta("expira");
    let (agora, ler) = relogio();
    let indice = IndiceLocal::em(raiz.join(NOME_DO_ARQUIVO)).com_relogio(ler);
    let sei = instalacao();
    assert_eq!(indice.entrada_por_id(&sei), None);
    indice.registrar_entrada_por_id(&sei, false);
    assert_eq!(indice.entrada_por_id(&sei), Some(false));
    agora.fetch_add(29 * DIA, Ordering::SeqCst);
    assert_eq!(indice.entrada_por_id(&sei), Some(false));
    agora.fetch_add(2 * DIA, Ordering::SeqCst);
    assert_eq!(indice.entrada_por_id(&sei), None);
    indice.registrar_entrada_por_id(&sei, false);
    assert_eq!(indice.entrada_por_id(&sei), Some(false));
}

#[test]
fn aceita_so_rebaixa_com_duas_recusas_seguidas() {
    let raiz = pasta("rebaixa");
    let indice = IndiceLocal::em(raiz.join(NOME_DO_ARQUIVO));
    let sei = instalacao();
    indice.registrar_entrada_por_id(&sei, true);
    assert_eq!(indice.entrada_por_id(&sei), Some(true));
    indice.registrar_entrada_por_id(&sei, false);
    assert_eq!(indice.entrada_por_id(&sei), Some(true));
    indice.registrar_entrada_por_id(&sei, true);
    indice.registrar_entrada_por_id(&sei, false);
    assert_eq!(indice.entrada_por_id(&sei), Some(true));
    indice.registrar_entrada_por_id(&sei, false);
    assert_eq!(indice.entrada_por_id(&sei), Some(false));
    let outra = Instalacao::interpretar("https://sei.exemplo/outro").unwrap();
    assert_eq!(indice.entrada_por_id(&outra), None);
}

#[test]
fn aceite_nao_expira() {
    let raiz = pasta("aceite");
    let (agora, ler) = relogio();
    let indice = IndiceLocal::em(raiz.join(NOME_DO_ARQUIVO)).com_relogio(ler);
    indice.registrar_entrada_por_id(&instalacao(), true);
    agora.fetch_add(400 * DIA, Ordering::SeqCst);
    assert_eq!(indice.entrada_por_id(&instalacao()), Some(true));
}

#[test]
fn debug_do_indice_nao_mostra_caminho_nem_dados() {
    let raiz = pasta("debug-indice");
    let indice = IndiceLocal::em(raiz.join(NOME_DO_ARQUIVO));
    let texto = format!("{indice:?}");
    assert!(texto.contains("ligado"), "{texto}");
    assert!(!texto.contains("ies-indice"), "{texto}");
}
