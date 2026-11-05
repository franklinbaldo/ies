---
type: ArchitectureDecision
title: Gravação privada única de arquivos locais
description: Uma peça, crate::arquivo_privado, cria ou substitui todo arquivo privado do IES com a mesma política; cada módulo só traduz a falha tipada para os seus subcódigos
status: accepted
decision: peca-unica-temporario-mais-ligacao-ou-rename
---

# Gravação privada única de arquivos locais

Refs [#211](https://github.com/franklinbaldo/ies/issues/211) (e a
duplicata [#216](https://github.com/franklinbaldo/ies/issues/216), que
contou os módulos e pediu a ordem depois da #207).

## Problema

Sete módulos repetiam `create_new(true)`, `0600`, `write_all`,
`sync_all` e limpeza, cada um com a sua política: `configuracao`
(sem `fsync` nem limpeza), `indice_local/abertura` (arquivo vazio e
diretório `0700`), `anexo_documento` e `foto_documento` (o gravador da
cópia depois da #207: parcial e `hard_link`), `arquivo_caixa` (só
`parent().is_dir()`), `arquivo_texto_documento` (confere links nos
ancestrais) e `sessao_login/gravacao` (temporário e `rename`). Não era
uma vulnerabilidade demonstrada; eram invariantes de segurança
espalhadas e divergentes.

## Alternativas

- **(a) manter as sete cópias** e alinhar a política à mão. Cada nova
  gravação copiaria a mais próxima; a divergência voltaria. Rejeitada.
- **(b) peça dentro de `transporte`**, ao lado da leitura
  `transporte::arquivo_privado`. A leitura está lá porque é dos arquivos
  de sessão do transporte; a escrita serve a exportação da caixa, a
  configuração, o índice e o `--saida`, nada disso transporte, e
  `transporte.rs` já passa de 12 KB. Rejeitada.
- **(c) `src/arquivo_privado.rs`** (escolhida): o módulo
  `crate::arquivo_privado` é o par de escrita da leitura
  `transporte::arquivo_privado` e não conhece nenhum formato, nome ou
  subcódigo. Os módulos continuam donos da serialização, dos nomes e dos
  erros públicos.
- **(d) arquivo temporário anônimo** (`O_TMPFILE` + `linkat`): nenhum
  parcial apareceria nem depois de um `kill -9`. Exige `libc` (a `std` não
  liga `linkat` com `AT_SYMLINK_FOLLOW` nem abre `O_TMPFILE` com nome
  vinculável), só existe no Linux e nem todo sistema de arquivos o
  aceita. Rejeitada; ver parciais órfãos abaixo.

## Política da peça

O chamador declara o modo em `Politica`:

| Garantia | `exclusiva()` | `substituicao()` |
|---|---|---|
| Destino existente (inclusive link) | `DestinoJaExiste`, nada muda | arquivo regular é trocado inteiro; link ou não regular é `DestinoNaoEhArquivoRegular` |
| Escrita | temporário `0600` no mesmo diretório, `sync_all` | igual |
| Publicação | `hard_link` para o nome final (falha se o nome surgiu) | `rename` sobre o nome final |
| Concorrência | dois criadores: um vence, o outro recebe `DestinoJaExiste` | o último `rename` vence; o lock é do chamador (sessão) |

Em ambos os modos:

- **Ancestrais.** O destino é tornado absoluto pelo diretório corrente
  (no Linux e no macOS `getcwd` já é o caminho físico) e cada ancestral,
  da raiz ao pai, é lido com `symlink_metadata`: link simbólico é
  `DiretorioLinkSimbolico`; ausente ou não diretório é
  `DiretorioAusente`. Com `criando_diretorios()` (só o índice) cada nível
  ausente nasce `0700`, um por vez, sem seguir link.
- **Limite.** `com_limite(n)` recusa com `AcimaDoLimite` antes de passar
  de `n` bytes.
- **Falha no meio.** Qualquer falha antes da publicação remove o
  temporário (no `Drop` do `Parcial`); o nome final nunca existe pela
  metade, nem no modo exclusivo, que antes escrevia direto no destino.
- **Diretório.** Depois da publicação, `sync_all` do diretório (melhor
  esforço, como já fazia a sessão).
- **Fora do Unix** não há modo de arquivo: vale a ACL do perfil, como na
  leitura; `cfg(unix)` só no modo. A conferência de ancestrais e a
  publicação são as mesmas.

### O que a peça não garante

A conferência dos ancestrais é por caminho: um processo do mesmo usuário
que troque um diretório por link entre a conferência e a criação não é
impedido (seria preciso `openat` por componente, fora da `std`). O
último componente nunca é seguido: `create_new` e `hard_link` não seguem
link, e `rename` substitui o link em vez de segui-lo. Sessão, índice ou
`--saida` sob um diretório que é link (por exemplo `/home` →
`/var/home`) passam a ser recusados: use o caminho físico.

### Parciais órfãos (o resto da #207)

O temporário chama-se `.NOME.parcial-PID-N` (N é um contador do
processo) no diretório do destino. Um `kill -9` ou queda de energia
entre a criação e a publicação não executa código nenhum, então o
parcial pode sobrar: isso não tem como ser evitado só com a `std`
(alternativa d). A peça reduz o dano: a próxima gravação do **mesmo**
nome remove os `.NOME.parcial-*` regulares com mais de uma hora sem
modificação (um parcial vivo é escrito ou publicado em segundos). Os
nomes antigos da #207 (`.DOC.vN.html.parcial-PID`) caem na mesma regra.
Parciais de outro nome nunca são tocados.

## Migração

| Módulo | Modo | Subcódigos mantidos |
|---|---|---|
| `configuracao::salvar_arquivos_de_acesso` | exclusiva | `configuracao_ja_existe_ou_sem_permissao` (destino, diretório, criação), `configuracao_nao_gravada` (escrita, publicação) |
| `indice_local/abertura` | exclusiva, criando diretórios | nenhum público: qualquer falha desliga o índice; destino já existente é o banco de outro `ies` |
| `anexo_documento` | `Parcial` exclusivo, publicado depois | `diretorio_do_anexo_ausente`, `diretorio_do_anexo_link_simbolico`, `arquivo_do_anexo_ja_existe`, `escrita_do_anexo_falhou` |
| `arquivo_caixa` | exclusiva | os quatro de antes e um novo, `diretorio_do_arquivo_da_caixa_link_simbolico` |
| `sessao_login/gravacao` | `Parcial` de substituição, um por arquivo, publicados em ordem | `destino_da_sessao_nao_eh_arquivo_regular` (também ancestral link), `gravacao_da_sessao_falhou` |
| `foto_documento` (cópia da #207) | exclusiva | `foto_divergente` (o nome surgiu), `gravacao_falhou` |
| `arquivo_texto_documento` | exclusiva | os cinco de antes |

O lock do login e o lock lateral do índice abrem com `create(true)` sem
conteúdo e ficam fora: não são gravações de dados. Nenhum comando ganhou
requisição.

## Verificação

- Testes da peça em `src/arquivo_privado/tests.rs`: arquivo novo `0600`,
  destino existente, link no destino e em ancestral, diretório ausente,
  diretórios `0700`, limite, falha no meio sem parcial, nome que surge
  antes da publicação, oito criadores exclusivos com um só vencedor,
  varredura só de parciais antigos do mesmo nome e
  `nenhuma_outra_gravacao_privada_usa_create_new` (o `grep` como teste).
- Por módulo: caixa, configuração, índice e sessão ganharam o teste do
  ancestral link; os testes existentes de cada módulo seguem iguais.

## Revisitar quando

- a `std` expuser `openat`/`linkat` com flags: conferir ancestrais por
  descritor e usar temporário anônimo;
- um sistema de arquivos suportado não tiver `hard_link` (o modo
  exclusivo falharia com `ConfirmacaoFalhou`).
