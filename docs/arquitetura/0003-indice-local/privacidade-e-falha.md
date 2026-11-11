---
type: Guide
title: Índice local — privacidade e modelo de falha
description: Privacidade, modelo de falha e auditoria complementar #188 do índice local
---

# Índice local — privacidade e modelo de falha

Parte do [ADR 0003 do índice local](../0003-indice-local.md), separada por responsabilidade.

## Privacidade

O arquivo revela quais processos a pessoa acessa e guarda URLs que
valem enquanto a sessão viver.

- Caminho: `IES_INDICE_FILE` se definido e não vazio; senão
  `ies.sqlite3` no diretório do `IES_URL_CONTROLE_FILE`
  (`ArquivosDeAcesso::url_controle`). `IES_SEM_INDICE` com valor não
  vazio diferente de `0` desliga.
- O diretório é criado `0700`; diretório existente com bits de grupo ou
  outros desliga o índice (o IES não muda o modo de diretórios que não
  criou). Link simbólico no caminho do diretório é aceito e o diretório
  real é o conferido ([gravação privada](../../decisoes/arquivos-privados/gravacao-privada.md#modelo-de-ameaça)). O banco nasce `0600` com `O_CREAT|O_EXCL` (não segue link simbólico); banco, `-wal` ou
  `-shm` que seja link simbólico, não seja arquivo regular ou tenha
  bits de grupo/outros desliga o índice. O SQLite abre com
  `SQLITE_OPEN_NOFOLLOW` e sem `CREATE`; `-wal` e `-shm` herdam o modo.
- URL assinada só sai por `UrlAssinada::url()`; `Debug` de
  `UrlAssinada`, `RegistroRecusado` e `IndiceLocal` não mostra URL,
  hash, geração nem cookie. O índice não escreve logs nem devolve o
  erro do SQLite.
- Apagar o arquivo é sempre seguro: tudo volta na próxima leitura.

## Modelo de falha

Arquivo corrompido, permissão, disco cheio, versão futura ou lock
além do `busy_timeout` **nunca** derrubam um comando: a primeira falha
desliga o índice daquele processo, e as consultas passam a devolver
`None` e as gravações a não fazer nada. A API não propaga erro do banco,
mas não o descarta (#786): `IndiceLocal` guarda cada falha como
`FalhaDoIndice` (`Banco` com o `ErroAberturaDoIndice` inteiro,
`RegistroRecusado` da entrada inválida, `EstadoDoAceiteForaDoConjunto`), e
a CLI a publica em `avisos` (JSON, omitida quando vazia) ou numa linha
`Aviso do índice` em stderr. A forma publicada tem o nome da variante, o
`kind` do sistema ou o código estendido do SQLite, nunca o caminho nem a
mensagem do SQLite. `IES_SEM_INDICE` não é falha. `entrada_por_id` devolve
`Option<AceiteDaEntrada>`: `None` é só "sem registro vigente".

- Cada operação abre a conexão, roda **uma transação minúscula** e
  fecha. Nenhuma transação atravessa HTTP: o chamador consulta, faz a
  requisição e só depois registra.
- `journal_mode=WAL` e `busy_timeout` de 2 s permitem vários `ies` ao
  mesmo tempo no mesmo banco.

Os testes offline de `src/indice_local/` cobrem isolamento por geração,
unidade e origem, expiração de recusas, aceite independente das duas formas
de entrada, contestação das identidades pelo servidor, migração, corrupção e
versão futura. A concorrência entre processos é exercitada por duas instâncias
do próprio executável de testes gravando ao mesmo tempo no mesmo banco; nenhum
desses cenários faz requisições ao SEI.

## Auditoria complementar #188 (testes offline)

A auditoria do índice em 05/11/2026 distinguiu **validade de geração**
e **validade temporal de uma autorização do servidor**. A chave
`(origem, geracao_sessao, unidade, acao, objeto)` impede devolver
qualquer link de geração anterior ou outra unidade. Um link pode
deixar de valer **dentro da mesma geração**; não existe TTL observado
ou relógio confiável da assinatura na tabela. O índice não pode
antecipar uma expiração desconhecida: o chamador só descobre a recusa
na primeira tentativa, esquece o palpite e prossegue conforme sua
própria política, sem reutilizar o link recusado nem repetir uma
escrita. O teste `link_expirado_recusado_pelo_servidor_nao_volta_do_cache`
simula exatamente essa recusa, seguida de falha no caminho alternativo,
e exige que o link antigo não retorne.

**O cache não autoriza escritas.** Os testes de integração
`indice_com_identidade_quente_nao_confirma_conclusao_sem_releitura_remota`
e `indice_aquecido_nao_autoriza_envio_sem_situacao_remota` aquecem
a identidade local, recebem respostas sintéticas de escrita e provam
que a situação remota prevalece. O teste
`indice_quente_nao_confirma_assinatura_ausente_na_arvore_relida`
exige a caneta na árvore posterior, não o palpite. O teste
`trocar_depende_do_controle_novo_nao_de_estado_local` comprova
a leitura do Controle da nova unidade antes de declarar a troca.
Todos usam transporte injetável, sem SEI real, sem retry e sem
dados de sessão persistidos em fixture nova.

**Desempenho e orçamento.** As sequências de requisições desses
testes são determinísticas e conferidas por roteiro. O CI padrão
roda a suíte Rust e os gates canônicos, mas não possui bancada
pareada reproduzível de latência SQLite com amostragem, warm-up
e controle do host. Assim, **benchmark de latência: NOT_RUN**.
É incorreto apresentar tempo local do runner ou comparar operação
sem índice com consulta que toca a rede como se fossem medidas
pareadas. A próxima medição, se necessária, deve usar cenários
offline idênticos de banco aquecido/indisponível, instrumentar
p50/p95 e manter as requisições constantes.
