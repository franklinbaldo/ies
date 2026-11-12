---
type: ArchitectureDecision
title: Gravação privada única de arquivos locais
description: Uma peça, crate::arquivo_privado, cria ou substitui todo arquivo privado do IES com a mesma política (nome final nunca seguido; diretório real não gravável por outros só nos caminhos que o IES gerencia); cada módulo só traduz a falha tipada para os seus subcódigos
status: accepted
decision: peca-unica-temporario-mais-ligacao-ou-rename
---

# Gravação privada única de arquivos locais

Refs [#211](https://github.com/franklinbaldo/ies/issues/211) (e a
duplicata [#216](https://github.com/franklinbaldo/ies/issues/216), que
contou os módulos e pediu a ordem depois da #207). Revisada em
05/11/2026 por duas decisões do mantenedor: os ancestrais deixaram de
ser recusados por serem links, e só os caminhos que o IES gerencia têm
o modo do diretório conferido ([modelo de ameaça](#modelo-de-ameaça)).

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
  subcódigo. Desde 12/11/2026 a leitura mora no mesmo módulo
  (`arquivo_privado::ler`, com `ErroArquivoPrivado`). Os módulos
  continuam donos da serialização, dos nomes e dos
  erros públicos.
- **(d) arquivo temporário anônimo** (`O_TMPFILE` + `linkat`): nenhum
  parcial apareceria nem depois de um `kill -9`. Exige `libc` (a `std` não
  liga `linkat` com `AT_SYMLINK_FOLLOW` nem abre `O_TMPFILE` com nome
  vinculável), só existe no Linux e nem todo sistema de arquivos o
  aceita. Rejeitada; ver parciais órfãos abaixo.

## Política da peça

O chamador declara o modo e, sempre, o dono do diretório: `exclusiva()`
e `substituicao()` só viram `Politica` com `.diretorio_do_ies()` ou
`.diretorio_do_usuario()` (não há padrão implícito; o compilador recusa
o chamador que não declara).

| Garantia | `exclusiva()` | `substituicao()` |
|---|---|---|
| Destino existente (inclusive link) | `DestinoJaExiste`, nada muda | arquivo regular é trocado inteiro; link ou não regular é `DestinoNaoEhArquivoRegular` |
| Escrita | temporário `0600` no mesmo diretório, `sync_all` | igual |
| Publicação | `hard_link` para o nome final (falha se o nome surgiu) | `rename` sobre o nome final |
| Concorrência | dois criadores: um vence, o outro recebe `DestinoJaExiste` | o último `rename` vence; o lock é do chamador (sessão) |

Em ambos os modos:

- **Nome final.** Nunca seguido: `create_new` e `hard_link` falham se o
  nome existe (inclusive link pendente), e `rename` substitui o link em
  vez de escrever através dele.
- **Diretório do destino.** O pai do destino (absoluto pelo diretório
  corrente) tem de existir e ser diretório, senão `DiretorioAusente`;
  links no caminho, inclusive o próprio `DIR` dado pela pessoa, são
  aceitos. O pai é resolvido com `fs::canonicalize` e o **diretório
  real** é usado para o temporário e o nome final.
- **Modo do diretório, só em `diretorio_do_ies()`.** No Unix o diretório
  real gravável por grupo ou outros (`mode & 0o022`) sem sticky bit
  (`0o1000`) é `DiretorioGravavelPorOutros`; ancestrais não são
  conferidos. Em `diretorio_do_usuario()` o modo não é conferido. Fora
  do Unix não há modo e a conferência não se aplica. Com
  `criando_diretorios()` (só o índice) cada nível ausente nasce `0700`,
  um por vez, num diretório que passa pela mesma conferência.
- **Limite.** `com_limite(n)` recusa com `AcimaDoLimite` antes de passar
  de `n` bytes.
- **Falha no meio.** Qualquer falha antes da publicação remove o
  temporário (no `Drop` do `Parcial`); o nome final nunca existe pela
  metade, nem no modo exclusivo, que antes escrevia direto no destino.
- **Diretório.** Depois da publicação, `sync_all` do diretório (melhor
  esforço, como já fazia a sessão).
- **Fora do Unix** não há modo de arquivo: vale a ACL do perfil, como na
  leitura; `cfg(unix)` só no modo e na conferência do diretório. A
  publicação é a mesma.

### Modelo de ameaça

A ameaça é a clássica do link plantado (*symlink/clobber*): outro
usuário que pode escrever no diretório do destino planta, no nome que o
IES vai usar, um link para um arquivo da vítima (ou para um nome que ele
vai ler depois) e espera o IES escrever através dele. Duas regras
fecham isso:

1. o nome final nunca é seguido, em nenhum modo e em nenhum diretório
   (acima);
2. nos caminhos que o **IES gerencia** (os arquivos da sessão e o
   diretório deles, o índice local, o arquivo de configuração e o
   futuro cache de documentos em `XDG_CACHE_HOME/ies`), o diretório
   real não pode ser gravável por mais ninguém, exceto com sticky bit
   (`/tmp`), onde outro usuário não remove nem troca o que é nosso e a
   criação exclusiva não segue o que ele plantou.

O IES policia só o que gerencia. Os diretórios de saída que a pessoa
escolhe (`--saida DIR` de documentos, fotos e anexos; o arquivo da
exportação da caixa) são dela: o IES não decide quem mais pode escrever
ali, e um `DIR` compartilhado com o grupo é um uso legítimo. Nesses, só
valem as garantias de correção: criação exclusiva (temporário +
`hard_link`, nunca sobrescreve) e o nome final nunca seguido.

Ancestrais não são ameaça nesse modelo: quem pode trocar um ancestral
por link já pode escrever no diretório que o contém, e o diretório real
é conferido depois de resolvido. Os arquivos da sessão (cookie, URL,
origem e o marcador `.geracao`) continuam lidos por
`arquivo_privado::ler`, que recusa link no nome final; as outras
leituras não mudaram.

### Alternativa rejeitada: conferir o modo de todo diretório de destino

A revisão anterior desta decisão (commit `7945480`) conferia o modo do
diretório real em **toda** gravação, inclusive no `--saida DIR` e na
exportação da caixa, com um subcódigo por módulo
(`diretorio_do_anexo_gravavel_por_outros`,
`diretorio_do_arquivo_da_caixa_gravavel_por_outros`,
`diretorio_texto_documento_gravavel_por_outros`). Rejeitada pelo
mantenedor em 05/11/2026: recusava diretórios legítimos da pessoa (um
`DIR` criado com `umask 002`, comum com grupo privado do usuário, ou
compartilhado de propósito com a equipe), e o modo de um diretório que
a pessoa escolheu é responsabilidade dela, não do IES. A proteção que
importa nesses diretórios (o nome final) não depende do modo.

### Alternativa rejeitada: recusar ancestral link (#211, até 05/11/2026)

A primeira versão desta peça lia cada ancestral, da raiz ao pai, com
`symlink_metadata` e recusava qualquer link
(`DiretorioLinkSimbolico`). Rejeitada pelo mantenedor em 05/11/2026:

- quebra instalações legítimas: no macOS `/var`, `/tmp` e `/etc` são
  links para `/private/...` e o `$TMPDIR` fica sob `/var/folders`; no
  Fedora Silverblue/Kinoite `/home` é link para `/var/home`. Nesses
  sistemas o `ies login` faria todas as requisições e falharia ao
  gravar a sessão;
- não fecha a corrida que dizia fechar: a conferência era por caminho,
  e um diretório pode ser trocado por link entre ela e a criação (seria
  preciso `openat` por componente, fora da `std`); esta decisão já
  registrava esse limite;
- não protege contra a ameaça real, que é o nome final e o diretório
  gravável por outros.

### O que a peça não garante

A conferência do diretório é por caminho: um processo do **mesmo
usuário**, ou de quem pode escrever num ancestral do diretório real,
que troque o diretório entre a conferência e a criação não é impedido.
Só o diretório imediato é conferido, não os ancestrais dele, e só nos
caminhos do IES. Num diretório da pessoa gravável por outros, quem pode
escrever ali pode apagar ou trocar o arquivo depois de publicado; o IES
só garante que não escreveu através de um link nem sobrescreveu nada.
Um diretório da sessão criado com `umask 002` é gravável pelo grupo e é
recusado: use `chmod g-w`.

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

| Módulo | Modo e diretório | Subcódigos |
|---|---|---|
| `configuracao::salvar_arquivos_de_acesso` | exclusiva, do IES | `configuracao_ja_existe_ou_sem_permissao` (destino, diretório ausente ou gravável por outros, criação), `configuracao_nao_gravada` (escrita, publicação) |
| `indice_local/abertura` | exclusiva, do IES, criando diretórios | nenhum público: qualquer falha, inclusive diretório gravável por outros, desliga o índice |
| `sessao_login/gravacao` | `Parcial` de substituição, do IES, um por arquivo, publicados em ordem | `destino_da_sessao_nao_eh_arquivo_regular` (só o nome final), `diretorio_da_sessao_gravavel_por_outros`, `gravacao_da_sessao_falhou` |
| `anexo_documento` | `Parcial` exclusivo, do usuário | `diretorio_do_anexo_ausente`, `arquivo_do_anexo_ja_existe`, `escrita_do_anexo_falhou` |
| `arquivo_caixa` | exclusiva, do usuário | os quatro de antes da #211 |
| `foto_documento` (cópia da #207) | exclusiva, do usuário | `foto_local_divergente` (o nome surgiu), `gravacao_foto_falhou`; o `DIR` como no texto |
| `arquivo_texto_documento` | exclusiva, do usuário | `destino_texto_documento_sem_nome`, `diretorio_texto_documento_ausente`, `arquivo_texto_documento_ja_existe`, `escrita_texto_documento_falhou` |

Só a sessão ganhou subcódigo novo: é o único caminho do IES com
subcódigos próprios de diretório; a configuração já tinha um que cobre
o caso e o índice não tem subcódigo público. Nos módulos do usuário a
peça nunca devolve `DiretorioGravavelPorOutros`.

O lock do login e o lock lateral do índice abrem com `create(true)` sem
conteúdo e ficam fora: não são gravações de dados. Nenhum comando ganhou
requisição.

## Verificação

- Testes da peça em `src/arquivo_privado/tests.rs`: arquivo novo `0600`,
  destino existente, link no destino (inclusive pendente e plantado
  depois de abrir o parcial, nos dois modos e nos dois donos de
  diretório), ancestral link aceito (`/home` → `/var/home` simulado),
  `DIR` link aceito, diretório do IES gravável por grupo ou outros
  (`0770`, `0730`, `0707`, `0702`, `0777`) recusado e sticky aceito,
  diretório do usuário `0775` e `0777` aceito, diretório ausente,
  diretórios `0700`, limite, falha no meio sem parcial, nome que surge
  antes da publicação, oito criadores exclusivos com um só vencedor,
  varredura só de parciais antigos do mesmo nome e
  `nenhuma_outra_gravacao_privada_usa_create_new` (o `grep` como teste).
- Por módulo: sessão, configuração e índice recusam o diretório
  gravável por outros (`0770`, `0707`, `0777`; sessão e configuração
  aceitam o sticky); caixa, texto, anexo e foto aceitam o diretório por
  link e o gravável pelo grupo ou por todos; o retrato da sessão (#222)
  testa a recusa de link no nome final de cada arquivo.

## Revisitar quando

- a `std` expuser `openat`/`linkat` com flags: conferir ancestrais por
  descritor e usar temporário anônimo;
- um sistema de arquivos suportado não tiver `hard_link` (o modo
  exclusivo falharia com `ConfirmacaoFalhou`).
