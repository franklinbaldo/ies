---
type: CommandContract
title: Listar processos da unidade
operation: unidade.listar_processos
command: ies unidade listar-processos
effect: read
implementation: partial
source_commit: 3d4a38c3b5987e4a8fc7f97ded188c4876ed77cd
---

# Listar processos da unidade

Lê os processos recebidos e gerados da unidade ativa na tela
**Controle de Processos** do SEI Web. A CaixaDeEntrada é da unidade;
sessão e credenciais são meios de acesso, não entidades proprietárias.

## Configuração e limites

A sessão existente é fornecida por caminhos locais definidos
em variáveis de ambiente, **nunca por valores secretos na linha de comando**:

- `IES_URL_CONTROLE_FILE`: caminho privado contendo URL HTTPS assinada
  `procedimento_controlar` publicada pela instalação autorizada.
- `IES_COOKIE_FILE`: caminho privado com o cookie de sessão existente.
- `IES_ORIGEM_FILE`: caminho privado com a origem HTTPS autorizada.

Os arquivos devem respeitar as regras de privacidade do adaptador
(no Unix, modo 0600). O IES confere a origem antes de ler cookies e
não segue redirects, não executa login, POST de paginação, nem realiza
escritas ou tentativa de outro backend. Não há configuração persistente
nem automação de autenticação.

`--unidade SIGLA` exige que a página identifique a unidade esperada.

### Exportar a caixa observada

`ies unidade listar-processos --saida CAMINHO` grava **o mesmo JSON**
que a apresentação normal produz, mas somente após conferir a cobertura
integral de recebidos e gerados. Um arquivo parcial jamais pode ser
publicado como caixa integral. Esta opção não acrescenta chamadas HTTP.

O destino pode ser relativo à pasta de trabalho ou absoluto. O diretório
pai precisa existir; o IES não cria diretórios, não sobrescreve arquivos
e não segue uma referência de destino preexistente, inclusive symlink.
A escrita é preparada em arquivo temporário privado na mesma pasta,
sincronizada e publicada de modo exclusivo, sem substituição. Em Unix,
o arquivo é criado com permissão `0600`. Em erro, a tentativa de
arquivo temporário é removida quando possível; nenhuma caixa incompleta
é anunciada como salva. O IES não imprime o JSON novamente quando
`--saida` foi escolhido: em `--json`, informa `arquivo` e
`salvo: true`; em modo humano, o caminho gravado.

Erros de destino existente, diretório pai ausente, preparação, gravação,
sincronização e publicação são **tipados** e possuem subcódigos estáveis.
Destino claramente existente ou pasta ausente falha antes do único GET
de leitura da caixa. Essa exportação é uma escrita **local no disco**,
não uma operação de escrita no SEI. A política de autorização para o
GET Web, a verificação de unidade e a recusa de paginação incompleta
continuam idênticas às da consulta sem `--saida`.

## Resultado e alcance

A resposta JSON segue o [golden v2](../../tests/fixtures/caixa-v2.json):
`unidade`, `origem`, `recebidos`, `gerados`; cada agrupamento tem
`processos` e `total_declarado`, sem campo `completa`.
A apresentação não transforma critérios de paginação em propriedades
do domínio.

O comando de domínio só retorna sucesso quando as tabelas disponíveis
cobrem os totais declarados. Se a tela exige outras páginas, ele
retorna `paginas_do_controle_nao_obtidas` e **não publica um resultado
parcial como se fosse toda a caixa**. Este recorte não implementa o
POST de paginação; portanto, algumas unidades ainda não podem ser
consultadas integralmente.

O replay de [evidências offline](../../docs/replay.md) continua em
`ies replay CENARIO`, independente de sessão e sem afirmar snapshot
transacional. SOAP e REST precisam de validação própria, sem presunção
de equivalência à tela Web.

## Evidências e verificação

Veja a [matriz da operação](../evidencias/unidade-listar_processos.md)
e os [testes Rust](../../tests/replay_unidade.rs). Nenhuma URL
assinada nem cookie pode aparecer em logs ou no resultado JSON.
